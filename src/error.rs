use std::fmt;
use std::path::Path;
use std::path::PathBuf;

use failure::{Backtrace, Context, Fail};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
/// An error that can occur while running a buildpack.
pub struct Error {
    ctx: Context<ErrorKind>,
}

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        self.ctx.get_context()
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.ctx.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.ctx.backtrace()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.ctx.fmt(f)
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(ctx: Context<ErrorKind>) -> Error {
        Error { ctx }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error::from(Context::new(kind))
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::from(ErrorKind::Io(err))
    }
}

impl From<toml::ser::Error> for Error {
    fn from(err: toml::ser::Error) -> Error {
        Error::from(ErrorKind::TomlSer(err))
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Error {
        Error::from(ErrorKind::TomlDe(err))
    }
}

/// The specific kind of error that can occur.
#[derive(Debug)]
pub enum ErrorKind {
    /// An error that occurred while working with a file path.
    Path(PathBuf),
    /// An unexpected I/O error.
    Io(std::io::Error),
    /// Toml Serialization error.
    TomlSer(toml::ser::Error),
    /// Toml Deserialization error.
    TomlDe(toml::de::Error),
    /// Hints that destructuring should not be exhaustive.
    #[doc(hidden)]
    __Nonexhaustive,
}

impl ErrorKind {
    pub(crate) fn new_path<P: AsRef<Path>>(path: P) -> ErrorKind {
        ErrorKind::Path(path.as_ref().to_path_buf())
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorKind::Path(ref path) => write!(f, "{}", path.display()),
            ErrorKind::Io(ref err) => err.fmt(f),
            ErrorKind::TomlSer(ref err) => err.fmt(f),
            ErrorKind::TomlDe(ref err) => err.fmt(f),
            ErrorKind::__Nonexhaustive => panic!("invalid error"),
        }
    }
}
