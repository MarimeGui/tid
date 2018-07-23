use magic_number::MagicNumberCheckError;
use std::error::Error;
use std::fmt;
use std::io::Error as IOError;

#[derive(Debug)]
pub enum TIDImportError {
    MagicNumber(MagicNumberCheckError),
    IO(IOError),
    UnknownDataType(u8),
}

impl Error for TIDImportError {
    fn description(&self) -> &str {
        match *self {
            TIDImportError::IO(ref e) => e.description(),
            TIDImportError::MagicNumber(ref e) => e.description(),
            TIDImportError::UnknownDataType(_) => "Unknown data type",
        }
    }
}

impl fmt::Display for TIDImportError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TIDImportError::IO(ref e) => e.fmt(f),
            TIDImportError::MagicNumber(ref e) => e.fmt(f),
            TIDImportError::UnknownDataType(ref dt) => {
                write!(f, "{} is unknown as a Data Type", dt)
            }
        }
    }
}

impl From<IOError> for TIDImportError {
    fn from(e: IOError) -> TIDImportError {
        TIDImportError::IO(e)
    }
}

impl From<MagicNumberCheckError> for TIDImportError {
    fn from(e: MagicNumberCheckError) -> TIDImportError {
        TIDImportError::MagicNumber(e)
    }
}