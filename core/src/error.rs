use std::fmt::Write;

#[derive(Debug)]
pub enum Error {
    Cartridge(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Cartridge(err) => f.write_fmt(format_args!("cartridge error: {err}")),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;
