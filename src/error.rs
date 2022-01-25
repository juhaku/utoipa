use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    Serde(serde_json::Error),
    Io(std::io::Error),
    FromUtf8(std::string::FromUtf8Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serde(error) => write!(f, "serde: {}", error),
            Self::Io(error) => write!(f, "io: {}", error),
            Self::FromUtf8(error) => write!(f, "from utf8: {}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<std::io::Error> for Error {
    fn from(io: std::io::Error) -> Self {
        Self::Io(io)
    }
}

impl From<serde_json::Error> for Error {
    fn from(serde: serde_json::Error) -> Self {
        Self::Serde(serde)
    }
}
