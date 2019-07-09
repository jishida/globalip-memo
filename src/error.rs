use std::{error, fmt};

#[derive(Debug)]
pub struct Error {
    inner: Box<Inner>,
}

#[derive(Debug)]
struct Inner {
    source: ErrorSource,
    message: String,
}

#[derive(Debug)]
pub enum ErrorSource {
    None,
    Io(std::io::Error),
    Json(serde_json::Error),
    Http(reqwest::Error),
    Regex(regex::Error),
    Addr(std::net::AddrParseError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.inner.message.as_str())
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self.inner.source {
            ErrorSource::None => None,
            ErrorSource::Io(ref e) => Some(e),
            ErrorSource::Json(ref e) => Some(e),
            ErrorSource::Http(ref e) => Some(e),
            ErrorSource::Regex(ref e) => Some(e),
            ErrorSource::Addr(ref e) => Some(e),
        }
    }
}

impl Error {
    pub fn new<S>(message: S, source: ErrorSource) -> Self
    where
        S: Into<String>,
    {
        Error {
            inner: Box::new(Inner {
                message: message.into(),
                source: source,
            }),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

macro_rules! err {
    ($msg:expr) => {
        $crate::error::Error::new($msg, $crate::error::ErrorSource::None)
    };
    ($f:expr, $($arg:expr),+) => {
        err!(format!($f, $($arg,)+))
    };
}

macro_rules! err_io {
    ($e:expr, $msg:expr) => {
        $crate::error::Error::new($msg, $crate::error::ErrorSource::Io($e))
    };
    ($e:expr, $f:expr, $($arg:expr),+) => {
        err_io!($e, format!($f, $($arg,)+))
    };
}

macro_rules! err_json {
    ($e:expr, $msg:expr) => {
        $crate::error::Error::new($msg, $crate::error::ErrorSource::Json($e))
    };
    ($e:expr, $f:expr, $($arg:expr),+) => {
        err_json!($e, format!($f, $($arg,)+))
    };
}

macro_rules! err_http {
    ($e:expr, $msg:expr) => {
        $crate::error::Error::new($msg, $crate::error::ErrorSource::Http($e))
    };
    ($e:expr, $f:expr, $($arg:expr),+) => {
        err_http!($e, format!($f, $($arg,)+))
    };
}

macro_rules! err_regex {
    ($e:expr, $msg:expr) => {
        $crate::error::Error::new($msg, $crate::error::ErrorSource::Regex($e))
    };
    ($e:expr, $f:expr, $($arg:expr),+) => {
        err_regex!($e, format!($f, $($arg,)+))
    };
}

macro_rules! err_addr {
    ($e:expr, $msg:expr) => {
        $crate::error::Error::new($msg, $crate::error::ErrorSource::Addr($e))
    };
    ($e:expr, $f:expr, $($arg:expr),+) => {
        err_addr!($e, format!($f, $($arg,)+))
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_err() {
        let io_err = || io::Error::from(io::ErrorKind::NotFound);
        expect_err(err!("test message"), "test message", is_none);
        expect_err(err!("test message".to_owned()), "test message", is_none);
        expect_err(err!("{} {}", "test", "message"), "test message", is_none);
        expect_err(err_io!(io_err(), "test message"), "test message", is_io);
        expect_err(
            err_io!(io_err(), "test message {}", 5),
            "test message 5",
            is_io,
        );
    }

    fn expect_err<F>(err: Error, expect_msg: &str, check_source: F)
    where
        F: Fn(&ErrorSource) -> bool,
    {
        assert_eq!(format!("{}", err), expect_msg.to_owned());
        assert!(check_source(&err.inner.source));
    }

    fn is_none(source: &ErrorSource) -> bool {
        match source {
            ErrorSource::None => true,
            _ => false,
        }
    }

    fn is_io(source: &ErrorSource) -> bool {
        match source {
            ErrorSource::Io(..) => true,
            _ => false,
        }
    }
}
