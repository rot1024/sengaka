use clap::{crate_name, crate_version, App, Arg};
use sengaka;
use std::error::Error as StdError;
use std::fmt;
use std::fs::File;
use std::io::{
    stdin, stdout, BufRead, BufReader, BufWriter, Cursor, Error as IOError, Read, Seek, SeekFrom,
    Write,
};
use std::path::Path;
use std::process::exit;

fn main() {
    if let Err(err) = main2() {
        eprintln!("{}", err);
        exit(1);
    }
}

fn main2() -> Result<(), Error> {
    let defalut_value_str = sengaka::DELTA_DEFAULT.to_string();

    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about("CLI tool to make images a line drawing")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("FILE")
                .takes_value(true)
                .help("Input file path. If omit, standard input is used."),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("FILE")
                .takes_value(true)
                .help("Output file path. If omit, standard output is used."),
        )
        .arg(
            Arg::with_name("input format")
                .short("I")
                .long("if")
                .value_name("FORMAT")
                .takes_value(true)
                .required_unless("input")
                .help("Input image format. e.g. png, jpg, ..."),
        )
        .arg(
            Arg::with_name("output format")
                .short("O")
                .long("of")
                .value_name("FORMAT")
                .takes_value(true)
                .required_unless("output")
                .help("Output image format. e.g. png, jpg, ..."),
        )
        .arg(
            Arg::with_name("sigma")
                .short("d")
                .long("sigma")
                .default_value(&defalut_value_str),
        )
        .get_matches();

    let mut buff = Vec::new();
    let sout = stdout();

    let input = match matches.value_of("input") {
        None => {
            stdin().lock().read_to_end(&mut buff)?;
            Input::Binary(Cursor::new(&buff))
        }
        Some(i) => Input::File(BufReader::new(File::open(i)?)),
    };

    let mut output: Box<Write> = match matches.value_of("output") {
        None => Box::new(sout.lock()),
        Some(o) => Box::new(BufWriter::new(File::create(o)?)),
    };

    let iformat = match matches.value_of("input format") {
        Some(f) => f.to_string(),
        None => Path::new(matches.value_of("input").unwrap())
            .extension()
            .ok_or(Error::UnknownFormat)?
            .to_string_lossy()
            .to_ascii_lowercase(),
    };

    let oformat = match matches.value_of("output format") {
        Some(f) => f.to_string(),
        None => Path::new(matches.value_of("output").unwrap())
            .extension()
            .ok_or(Error::UnknownFormat)?
            .to_string_lossy()
            .to_ascii_lowercase(),
    };

    sengaka::sengaka(input, &mut output, &iformat, &oformat)?;

    output.flush()?;
    Ok(())
}

enum Input<'a> {
    Binary(Cursor<&'a [u8]>),
    File(BufReader<std::fs::File>),
}

impl<'a> Read for Input<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Input::Binary(v) => v.read(buf),
            Input::File(f) => f.read(buf),
        }
    }
}

impl<'a> BufRead for Input<'a> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        match self {
            Input::Binary(v) => v.fill_buf(),
            Input::File(f) => f.fill_buf(),
        }
    }

    fn consume(&mut self, amt: usize) {
        match self {
            Input::Binary(v) => v.consume(amt),
            Input::File(f) => f.consume(amt),
        }
    }
}

impl<'a> Seek for Input<'a> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match self {
            Input::Binary(v) => v.seek(pos),
            Input::File(f) => f.seek(pos),
        }
    }
}

#[derive(Debug)]
enum Error {
    UnknownFormat,
    IO(IOError),
    Sengaka(sengaka::Error),
    Misc(Box<StdError>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IO(e) => write!(f, "{}", e),
            Error::Misc(e) => write!(f, "{}", e),
            Error::Sengaka(e) => write!(f, "{}", e),
            Error::UnknownFormat => write!(f, "unknown format"),
        }
    }
}

impl From<IOError> for Error {
    fn from(e: IOError) -> Error {
        Error::IO(e)
    }
}

impl From<sengaka::Error> for Error {
    fn from(e: sengaka::Error) -> Error {
        Error::Sengaka(e)
    }
}

impl From<Box<StdError>> for Error {
    fn from(e: Box<StdError>) -> Error {
        Error::Misc(e)
    }
}
