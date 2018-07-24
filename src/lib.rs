extern crate ez_io;
extern crate magic_number;
extern crate rgb;

pub mod error;
pub mod texture_decode;

use error::TIDImportError;
use ez_io::ReadE;
use magic_number::check_magic_number;
use rgb::{FromSlice, RGBA8};
use std::io::{Cursor, Read, Seek, SeekFrom};
use texture_decode::{decode_bc1_block, morton_order};

#[derive(Clone)]
pub struct TID {
    pub data_type: DataType,
    pub dimensions: ImageSize,
    pub image_buffer: Vec<u8>,
}

#[derive(Clone, Copy)]
pub enum DataType {
    RGBA = 0x90,
    ARGB = 0x92,
    BC1_94 = 0x94,
    BC1_9C = 0x9C,
}

#[derive(Clone, Copy)]
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
        let mut image_buffer = vec![0u8; (file_size - 0x80) as usize];
        reader.read_exact(&mut image_buffer)?;
        Ok(TID {
            data_type,
            dimensions,
            image_buffer,
        })
    }
    pub fn convert(&self) -> Vec<RGBA8> {
        match self.data_type {
            DataType::RGBA => self.image_buffer.as_rgba().to_vec(),
            DataType::ARGB => {
                let mut image_out =
                    Vec::with_capacity((self.dimensions.width * self.dimensions.height) as usize);
                for i in (0..(self.image_buffer.len())).step_by(4) {
                    image_out.push(RGBA8 {
                        a: self.image_buffer[i],
                        r: self.image_buffer[i + 1],
                        g: self.image_buffer[i + 2],
                        b: self.image_buffer[i + 3],
                    });
                }
                image_out
            }
            DataType::BC1_94 | DataType::BC1_9C => {
                let mut image_out =
                    Vec::with_capacity((self.dimensions.width * self.dimensions.height) as usize);
                let reader = &mut Cursor::new(self.image_buffer.clone());
                for i in 0..((self.dimensions.width / 4) * self.dimensions.height / 4) {
                    let tile = decode_bc1_block(reader);
                    let tile_write_position = morton_order(i, self.dimensions);
                    for tile_y in 0..4 {
                        for tile_x in 0..4 {
                            let actual_pos_x = (tile_write_position.x * 4) + tile_x;
                            let actual_pos_y = (tile_write_position.y * 4) + tile_y;
                            image_out[((actual_pos_y * self.dimensions.width) + actual_pos_x)
                                          as usize] = tile[((tile_y * 4) + tile_x) as usize];
                        }
                    }
                }
                image_out
            }
        }
    }
}

impl DataType {
    fn import<R: Read>(reader: &mut R) -> Result<DataType, TIDImportError> {
        Ok(match reader.read_to_u8()? {
            0x90 => DataType::RGBA,
            0x92 => DataType::ARGB,
            0x94 => DataType::BC1_94,
            0x9C => DataType::BC1_9C,
            x => return Err(TIDImportError::UnknownDataType(x)),
        })
    }
}

impl ImageSize {
    fn import<R: Read>(reader: &mut R) -> Result<ImageSize, TIDImportError> {
        Ok(ImageSize {
            width: reader.read_le_to_u32()?,
            height: reader.read_le_to_u32()?,
        })
    }
}
