extern crate ez_io;
extern crate magic_number;

pub mod error;

use error::TIDImportError;
use ez_io::ReadE;
use magic_number::check_magic_number;
use std::io::{Read, Seek, SeekFrom};

pub struct TID {
    pub data_type: DataType,
    pub dimensions: ImageSize,
    pub image_buffer: Vec<u8>,
}

pub enum DataType {
    Rgba = 0x90,
    Argb = 0x92,
    Bc1A = 0x94,
    Bc1B = 0x9C,
}

pub struct ImageSize {
    pub width: u32,
    pub height: u32,
}

impl TID {
    pub fn import<R: Read + Seek>(reader: &mut R) -> Result<TID, TIDImportError> {
        check_magic_number(reader, vec![b'T', b'I', b'D'])?;
        let data_type = DataType::import(reader)?;
        let file_size = reader.read_le_to_u32()?;
        reader.seek(SeekFrom::Current(0x3C))?;
        let dimensions = ImageSize::import(reader)?;
        reader.seek(SeekFrom::Current(0x34))?;
        let mut image_buffer = vec![0u8; (file_size - 80) as usize];
        reader.read_exact(&mut image_buffer)?;
        Ok(TID {
            data_type,
            dimensions,
            image_buffer,
        })
    }
}

impl DataType {
    pub fn import<R: Read>(reader: &mut R) -> Result<DataType, TIDImportError> {
        Ok(match reader.read_to_u8()? {
            0x90 => DataType::Rgba,
            0x92 => DataType::Argb,
            0x94 => DataType::Bc1A,
            0x9C => DataType::Bc1B,
            x => return Err(TIDImportError::UnknownDataType(x)),
        })
    }
}

impl ImageSize {
    pub fn import<R: Read>(reader: &mut R) -> Result<ImageSize, TIDImportError> {
        Ok(ImageSize {
            width: reader.read_le_to_u32()?,
            height: reader.read_le_to_u32()?,
        })
    }
}
