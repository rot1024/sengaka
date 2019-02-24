use clap::{crate_name, crate_version, App, Arg};
use sengaka;
use std::error::Error as StdError;
use std::fmt;
use std::io::{Error as IOError, Write};
use std::path::PathBuf;
use std::process::exit;

mod io;

fn main() {
    if let Err(err) = main2() {
        eprintln!("{}", err);
        exit(1);
    }
}

fn main2() -> Result<(), Error> {
    let def_sigma_str = sengaka::SIGMA_DEFAULT.to_string();
    let def_shadow_str = sengaka::SHADOW_DEFAULT.to_string();

    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about("CLI tool to make images a line drawing")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("FILE")
                .takes_value(true)
                .help("Input file or directory path. If omit, stdin is used."),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("FILE")
                .takes_value(true)
                .help("Output file or directory path. If omit, stdout is used."),
        )
        .arg(
            Arg::with_name("input format")
                .short("I")
                .long("if")
                .value_name("FORMAT")
                .takes_value(true)
                .required_unless("input")
                .help("Input image format. Required if input is stdin. e.g. png, jpg, ..."),
        )
        .arg(
            Arg::with_name("output format")
                .short("O")
                .long("of")
                .value_name("FORMAT")
                .takes_value(true)
                .required_unless("output")
                .help("Output image format. Required if output is stdout. e.g. png, jpg, ..."),
        )
        .arg(
            Arg::with_name("sigma")
                .short("s")
                .long("sigma")
                .takes_value(true)
                .help("Blur sigma")
                .default_value(&def_sigma_str),
        )
        .arg(
            Arg::with_name("shadow")
                .short("S")
                .long("shadow")
                .takes_value(true)
                .help("Shadow input level (0 ~ 255)")
                .default_value(&def_shadow_str),
        )
        .arg(
            Arg::with_name("quite")
                .short("q")
                .long("quite")
                .help("Disables printing logs"),
        )
        .get_matches();

    let input = match matches.value_of("input") {
        None => io::Input::Stdin,
        Some(i) => io::Input::FileOrDir(PathBuf::from(i)),
    };
    let output = match matches.value_of("output") {
        None => io::Output::Stdout,
        Some(o) => io::Output::FileOrDir(PathBuf::from(o)),
    };
    let iformat = matches.value_of("input format").map(|s| s.to_string());
    let oformat = matches.value_of("output format").map(|s| s.to_string());
    let quite = matches.is_present("quite");

    let sigma = matches
        .value_of("sigma")
        .unwrap()
        .parse()
        .map_err(|_| Error::InvalidSigma)?;

    let shadow = matches
        .value_of("shadow")
        .unwrap()
        .parse()
        .map_err(|_| Error::InvalidShadow)?;

    let inputoutput = io::IO::new(input, output, iformat, oformat)?;

    for f in inputoutput.iter() {
        let mut item = f?;

        if !quite {
            if let Some(f) = item.file_name {
                eprintln!("{}", f);
            }
        }

        sengaka::sengaka(
            item.input,
            &mut item.output,
            item.input_format,
            item.output_format,
            sigma,
            shadow,
        )?;

        item.output.flush()?;
    }

    Ok(())
}

#[derive(Debug)]
enum Error {
    UnknownFormat,
    InvalidSigma,
    InvalidShadow,
    UnknownFileName,
    MultiToSingle,
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
            Error::InvalidSigma => write!(f, "invalid sigma"),
            Error::InvalidShadow => write!(f, "invalid shadow"),
            Error::UnknownFileName => write!(
                f,
                "stdin has no file name. specify file name in -o option or use stdout"
            ),
            Error::MultiToSingle => write!(f, "input is multiple but output is single"),
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

impl From<io::IOIteratorError> for Error {
    fn from(e: io::IOIteratorError) -> Self {
        match e {
            io::IOIteratorError::IO(err) => Error::from(err),
            io::IOIteratorError::UnknownInputFormat | io::IOIteratorError::UnknownOutputFormat => {
                Error::UnknownFormat
            }
            io::IOIteratorError::UnknownFileName => Error::UnknownFileName,
            io::IOIteratorError::MultiToSingle => Error::MultiToSingle,
        }
    }
}

impl From<io::IOItemError> for Error {
    fn from(e: io::IOItemError) -> Self {
        match e {
            io::IOItemError::InputIO(err) => Error::IO(err),
            io::IOItemError::OutputIO(err) => Error::IO(err),
        }
    }
}
