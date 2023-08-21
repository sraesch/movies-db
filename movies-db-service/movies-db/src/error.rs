use quick_error::quick_error;
use std::io;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        IO(err: String) {
            display("{}", err)
        }
        Internal(err: String) {
            display("{}", err)
        }
        InvalidArgument(err: String) {
            display("{}", err)
        }
        NotFound(err: String) {
            display("{}", err)
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IO(format!("{}", error))
    }
}

impl From<rusqlite::Error> for Error {
    fn from(error: rusqlite::Error) -> Self {
        Error::Internal(format!("SQLite Error: {}", error))
    }
}
