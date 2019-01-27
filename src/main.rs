use clap::{crate_name, crate_version, App, Arg};
use sengaka::DELTA_DEFAULT;
use std::error::Error as StdError;
use std::fmt;
use std::fs::File;
use std::io::{stdin, stdout, BufRead, BufReader, BufWriter, Error as IOError, Write};
use std::process::exit;

fn main() {
    if let Err(err) = main2() {
        eprintln!("{}", err);
        exit(1);
    }
}

fn main2() -> Result<(), Error> {
    let defalut_value_str = DELTA_DEFAULT.to_string();

    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about("CLI tool to make images a line drawing")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .default_value("-")
                .value_name("FILE")
                .help("Input file path. If \"-\" is set, standard input is used."),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .default_value("-")
                .value_name("FILE")
                .help("Output file path. If \"-\" is set, standard output is used."),
        )
        .arg(Arg::with_name("input format").long("if").takes_value(true))
        .arg(Arg::with_name("output format").long("of").takes_value(true))
        .arg(
            Arg::with_name("delta")
                .short("d")
                .long("delta")
                .default_value(&defalut_value_str),
        )
        .get_matches();

    let sin = stdin();
    let sout = stdout();

    let mut input: Box<BufRead> = match matches.value_of("input") {
        Some("-") | None => Box::new(sin.lock()),
        Some(i) => Box::new(BufReader::new(File::open(i)?)),
    };

    let mut output: Box<Write> = match matches.value_of("output") {
        Some("-") | None => Box::new(sout.lock()),
        Some(o) => Box::new(BufWriter::new(File::open(o)?)),
    };

    // just copy
    std::io::copy(&mut input, &mut output)?;

    output.flush()?;
    Ok(())
}

#[derive(Debug)]
enum Error {
    IO(IOError),
    Misc(Box<StdError>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IO(e) => write!(f, "{}", e),
            Error::Misc(e) => write!(f, "{}", e),
        }
    }
}

impl From<IOError> for Error {
    fn from(e: IOError) -> Error {
        Error::IO(e)
    }
}

impl From<Box<StdError>> for Error {
    fn from(e: Box<StdError>) -> Error {
        Error::Misc(e)
    }
}
