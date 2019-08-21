use failure::{Backtrace, Context, Fail};
use std::{
    fmt,
    path::{Path, PathBuf},
};

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
    fn cause(&self) -> Option<&dyn Fail> {
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

impl From<std::env::VarError> for Error {
    fn from(err: std::env::VarError) -> Error {
        Error::from(ErrorKind::Env(err))
    }
}

impl From<std::ffi::OsString> for Error {
    fn from(os_string: std::ffi::OsString) -> Error {
        Error::from(ErrorKind::OsString(os_string))
    }
}

/// The specific kind of error that can occur.
#[derive(Debug)]
pub enum ErrorKind {
    /// A path contains invalid unicode.
    PathUnicode(PathBuf),
    /// An unexpected I/O error.
    Io(std::io::Error),
    /// File Not Found I/o error.
    FileNotFound(PathBuf),
    /// Toml Serialization error.
    TomlSer(toml::ser::Error),
    /// Toml Deserialization error.
    TomlDe(toml::de::Error),
    /// Env Var fetching error.
    Env(std::env::VarError),
    /// OsString contains invalid Unicode data
    OsString(std::ffi::OsString),
    /// No
    NoArgs,
    /// Hints that destructuring should not be exhaustive.
    #[doc(hidden)]
    __Nonexhaustive,
}

impl ErrorKind {
    pub(crate) fn new_path<P: AsRef<Path>>(path: P) -> ErrorKind {
        ErrorKind::PathUnicode(path.as_ref().to_path_buf())
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorKind::PathUnicode(ref path) => {
                write!(f, "Path contains invalid unicode: {}", path.display())
            }
            ErrorKind::Io(ref err) => err.fmt(f),
            ErrorKind::FileNotFound(ref expected_file) => {
                write!(f, "File Not Found: {}", expected_file.display())
            }
            ErrorKind::TomlSer(ref err) => err.fmt(f),
            ErrorKind::TomlDe(ref err) => err.fmt(f),
            ErrorKind::Env(ref err) => err.fmt(f),
            ErrorKind::OsString(ref _os_string) => write!(f, "invalid unicode characters provided"),
            ErrorKind::NoArgs => write!(f, "Not enough args passed"),
            ErrorKind::__Nonexhaustive => panic!("invalid error"),
        }
    }
}
