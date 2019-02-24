use sengaka::detect_format;
use std::fs;
use std::fs::File;
use std::io::Result as IOResult;
use std::io::{
  stdin, stdout, BufRead, BufReader, BufWriter, Cursor, Read, Seek, SeekFrom, Stdout, StdoutLock,
  Write,
};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Input {
  Stdin,
  FileOrDir(PathBuf),
}

#[derive(Debug)]
pub enum Output {
  Stdout,
  FileOrDir(PathBuf),
}

pub enum Reader {
  Binary(Cursor<Vec<u8>>),
  File(BufReader<File>),
}

impl Read for Reader {
  fn read(&mut self, buf: &mut [u8]) -> IOResult<usize> {
    match self {
      Reader::Binary(r) => r.read(buf),
      Reader::File(r) => r.read(buf),
    }
  }
}

impl Seek for Reader {
  fn seek(&mut self, pos: SeekFrom) -> IOResult<u64> {
    match self {
      Reader::Binary(r) => r.seek(pos),
      Reader::File(r) => r.seek(pos),
    }
  }
}

impl BufRead for Reader {
  fn fill_buf(&mut self) -> IOResult<&[u8]> {
    match self {
      Reader::Binary(r) => r.fill_buf(),
      Reader::File(r) => r.fill_buf(),
    }
  }

  fn consume(&mut self, atm: usize) {
    match self {
      Reader::Binary(r) => r.consume(atm),
      Reader::File(r) => r.consume(atm),
    }
  }
}

pub enum Writer<'a> {
  Stdout(StdoutLock<'a>),
  File(BufWriter<File>),
}

impl<'a> Write for Writer<'a> {
  fn write(&mut self, buf: &[u8]) -> IOResult<usize> {
    match self {
      Writer::Stdout(w) => w.write(buf),
      Writer::File(w) => w.write(buf),
    }
  }

  fn flush(&mut self) -> IOResult<()> {
    match self {
      Writer::Stdout(w) => w.flush(),
      Writer::File(w) => w.flush(),
    }
  }
}

pub struct IOItem<'a> {
  pub input: Reader,
  pub output: Writer<'a>,
  pub input_format: &'a str,
  pub output_format: &'a str,
  pub file_name: Option<&'a str>,
}

#[derive(Debug)]
pub enum IOItemError {
  InputIO(std::io::Error),
  OutputIO(std::io::Error),
}

#[derive(Debug)]
pub enum IOIteratorError {
  MultiToSingle,
  UnknownFileName,
  UnknownInputFormat,
  UnknownOutputFormat,
  IO(std::io::Error),
}

impl From<std::io::Error> for IOIteratorError {
  fn from(e: std::io::Error) -> Self {
    IOIteratorError::IO(e)
  }
}

#[derive(Debug)]
enum IOIteratorInputPort {
  Stdin(String),
  Files(Vec<(PathBuf, String)>),
}

#[derive(Debug)]
enum IOIteratorOutputPort {
  Stdout(Stdout, String),
  Files(Vec<(PathBuf, String)>),
}

#[derive(Debug)]
pub struct IO {
  input: IOIteratorInputPort,
  output: IOIteratorOutputPort,
}

#[derive(Debug)]
pub struct IOIterator<'a> {
  input: &'a IOIteratorInputPort,
  output: &'a IOIteratorOutputPort,
  counter: usize,
}

impl IO {
  pub fn new(
    i: Input,
    o: Output,
    input_format: Option<String>,
    output_format: Option<String>,
  ) -> Result<Self, IOIteratorError> {
    if let (Input::Stdin, None) = (&i, &input_format) {
      return Err(IOIteratorError::UnknownInputFormat);
    };
    if let (Output::Stdout, None) = (&o, &output_format) {
      return Err(IOIteratorError::UnknownOutputFormat);
    };

    let i_dir = if let Input::FileOrDir(p) = &i {
      Some(is_dir(p)?)
    } else {
      None
    };
    let o_dir = if let Output::FileOrDir(p) = &o {
      Some(is_dir(p).or_else(|e| match e.kind() {
        std::io::ErrorKind::NotFound => fs::create_dir_all(p).map(|_| true),
        _ => Err(e),
      })?)
    } else {
      None
    };

    let (i_dir, o_dir) = match (i_dir, o_dir) {
      (Some(true), Some(false)) | (Some(true), None) => return Err(IOIteratorError::MultiToSingle),
      (None, Some(true)) => return Err(IOIteratorError::UnknownFileName),
      (id @ _, od @ _) => (id.unwrap_or(false), od.unwrap_or(false)),
    };

    let input = match &i {
      Input::FileOrDir(ref p) => {
        if i_dir {
          IOIteratorInputPort::Files(get_file_paths(p)?)
        } else if let Some(ext) = get_extension(p) {
          IOIteratorInputPort::Files(vec![(p.to_owned(), ext)])
        } else {
          return Err(IOIteratorError::UnknownOutputFormat);
        }
      }
      Input::Stdin => IOIteratorInputPort::Stdin(input_format.unwrap()),
    };

    let output = match &o {
      Output::FileOrDir(ref p) => match &input {
        IOIteratorInputPort::Files(ref ip) => {
          if o_dir {
            IOIteratorOutputPort::Files(
              ip.iter()
                .map(|(q, r)| (p.join(q.file_name().unwrap()), r.to_string()))
                .collect(),
            )
          } else if let Some(ext) = get_extension(p) {
            IOIteratorOutputPort::Files(vec![(p.to_owned(), ext)])
          } else {
            return Err(IOIteratorError::UnknownOutputFormat);
          }
        }
        IOIteratorInputPort::Stdin(_) => {
          if is_dir(p)? {
            return Err(IOIteratorError::UnknownFileName);
          } else if let Some(ext) = get_extension(p) {
            IOIteratorOutputPort::Files(vec![(p.to_owned(), ext)])
          } else {
            return Err(IOIteratorError::UnknownOutputFormat);
          }
        }
      },
      Output::Stdout => IOIteratorOutputPort::Stdout(stdout(), output_format.unwrap()),
    };

    Ok(Self {
      input: input,
      output: output,
    })
  }

  pub fn iter(&self) -> IOIterator {
    IOIterator {
      input: &self.input,
      output: &self.output,
      counter: 0,
    }
  }
}

impl<'a> Iterator for IOIterator<'a> {
  type Item = Result<IOItem<'a>, IOItemError>;

  fn next(&mut self) -> Option<<Self as Iterator>::Item> {
    let i = self.counter;
    self.counter += 1;

    let input = match &(self.input) {
      IOIteratorInputPort::Stdin(ext) => {
        if i == 0 {
          let mut buf = Vec::new();
          if let Err(err) = stdin().lock().read_to_end(&mut buf) {
            return Some(Err(IOItemError::InputIO(err)));
          }
          (Reader::Binary(Cursor::new(buf)), ext, None)
        } else {
          return None;
        }
      }
      IOIteratorInputPort::Files(paths) => {
        let inp = if let Some(inp) = paths.get(i) {
          inp
        } else {
          return None;
        };
        (
          match File::open(&inp.0) {
            Ok(r) => Reader::File(BufReader::new(r)),
            Err(err) => return Some(Err(IOItemError::InputIO(err))),
          },
          &inp.1,
          inp.0.file_name().and_then(|f| f.to_str()),
        )
      }
    };

    let output = match &(self.output) {
      IOIteratorOutputPort::Stdout(out, ext) => {
        if i == 0 {
          (Writer::Stdout(out.lock()), ext)
        } else {
          return None;
        }
      }
      IOIteratorOutputPort::Files(paths) => {
        let out = if let Some(out) = paths.get(i) {
          out
        } else {
          return None;
        };

        (
          match File::create(&out.0) {
            Ok(w) => Writer::File(BufWriter::new(w)),
            Err(err) => return Some(Err(IOItemError::OutputIO(err))),
          },
          &out.1,
        )
      }
    };

    Some(Ok(IOItem {
      input: input.0,
      output: output.0,
      input_format: input.1,
      output_format: output.1,
      file_name: input.2,
    }))
  }
}

fn get_file_paths(p: &Path) -> IOResult<Vec<(PathBuf, String)>> {
  let mut paths = Vec::new();
  for r in fs::read_dir(p)? {
    let entry = r?;
    if !entry.file_type()?.is_dir() {
      let entry_path = entry.path();
      let ext = entry_path
        .extension()
        .and_then(|e| e.to_str())
        .filter(|e| detect_format(e).is_some());
      if let Some(ext) = ext {
        paths.push((entry.path(), ext.to_string()));
      }
    }
  }
  return Ok(paths);
}

fn is_dir(p: &Path) -> IOResult<bool> {
  Ok(fs::metadata(p)?.is_dir())
}

fn get_extension(p: &Path) -> Option<String> {
  p.extension()
    .map(|e| e.to_string_lossy().to_ascii_lowercase())
    .filter(|e| detect_format(&e).is_some())
}
