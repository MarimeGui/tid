use ez_io::error::MagicNumberCheckError;
use std::error::Error;
use std::fmt;
use std::io::Error as IOError;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum TIDError {
    MagicNumber(MagicNumberCheckError),
    IO(IOError),
    UnknownDataType(u8),
    UnknownFourCC(Vec<u8>),
    NoFourCC,
    NameDecodeError(FromUtf8Error),
}

impl Error for TIDError {
    fn description(&self) -> &str {
        match *self {
            TIDError::IO(ref e) => e.description(),
            TIDError::MagicNumber(ref e) => e.description(),
            TIDError::UnknownDataType(_) => "Unknown data type",
            TIDError::UnknownFourCC(_) => "Unknown FourCC code",
            TIDError::NoFourCC => "Missing FourCC for BC type",
            TIDError::NameDecodeError(_) => "Failed to read name in header",
        }
    }
}

impl fmt::Display for TIDError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TIDError::IO(ref e) => e.fmt(f),
            TIDError::MagicNumber(ref e) => e.fmt(f),
            TIDError::UnknownDataType(ref dt) => write!(f, "{} is unknown as a Data Type", dt),
            TIDError::UnknownFourCC(ref fcc) => write!(f, "{:?} is unknown", fcc),
            TIDError::NoFourCC => write!(
                f,
                "No FourCC was defined, cannot infer what type of Block Compression to decode"
            ),
            TIDError::NameDecodeError(ref e) => e.fmt(f),
        }
    }
}

impl From<IOError> for TIDError {
    fn from(e: IOError) -> TIDError {
        TIDError::IO(e)
    }
}

impl From<MagicNumberCheckError> for TIDError {
    fn from(e: MagicNumberCheckError) -> TIDError {
        TIDError::MagicNumber(e)
    }
}

impl From<FromUtf8Error> for TIDError {
    fn from(e: FromUtf8Error) -> TIDError {
        TIDError::NameDecodeError(e)
    }
}
