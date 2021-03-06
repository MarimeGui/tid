extern crate ez_io;
extern crate rgb;

pub mod error;
pub mod texture_decode;

use crate::error::TIDError;
use crate::texture_decode::{decode_bc1_block, morton_order};
use ez_io::{MagicNumberCheck, ReadE};
use rgb::{FromSlice, RGBA8};
use std::fmt::{Display, Formatter, Result as FMTResult};
use std::io::{Read, Seek, SeekFrom};

pub type Result<T> = ::std::result::Result<T, TIDError>;

#[derive(Clone)]
pub struct TID {
    pub file_size: u32,
    pub data_type: DataType,
    pub name: String,
    pub dimensions: ImageSize,
    pub bc_type: BlockCompressionType,
}

#[derive(Clone, Copy)]
pub enum DataType {
    BlockCompression,
    RGBA,
    ARGB,
}

#[derive(Clone, Copy)]
pub enum BlockCompressionType {
    None,
    DXT1,
    DXT5,
}

#[derive(Clone, Copy)]
pub struct ImageSize {
    pub width: u32,
    pub height: u32,
}

impl TID {
    pub fn import<R: Read + Seek>(reader: &mut R) -> Result<TID> {
        reader.check_magic_number(&[b'T', b'I', b'D'])?;
        let data_type = DataType::import(reader)?;
        let file_size = reader.read_le_to_u32()?;
        reader.seek(SeekFrom::Start(0x20))?;
        let name = {
            let mut buf = vec![0u8; 32usize];
            reader.read_exact(&mut buf)?;
            let mut buf_keep = Vec::new();
            for c in buf {
                if c != 0 {
                    buf_keep.push(c);
                }
            }
            String::from_utf8(buf_keep)?
        };
        reader.seek(SeekFrom::Start(0x44))?;
        let dimensions = ImageSize::import(reader)?;
        reader.seek(SeekFrom::Start(0x64))?;
        let bc_type = BlockCompressionType::import(reader)?;
        reader.seek(SeekFrom::Start(0x80))?;
        Ok(TID {
            file_size,
            data_type,
            name,
            dimensions,
            bc_type,
        })
    }
    pub fn convert<R: Read>(&self, reader: &mut R) -> Result<Vec<RGBA8>> {
        match self.data_type {
            DataType::RGBA => {
                let buffer_size = self.dimensions.width * self.dimensions.height * 4;
                let mut image_buffer = vec![0u8; buffer_size as usize];
                reader.read_exact(&mut image_buffer)?;
                Ok(image_buffer.as_rgba().to_vec())
            }
            DataType::ARGB => {
                let buffer_size = (self.dimensions.width * self.dimensions.height * 4) as usize;
                let mut image_out =
                    Vec::with_capacity((self.dimensions.width * self.dimensions.height) as usize);
                for i in (0..buffer_size).step_by(4) {
                    let mut raw_pixel = vec![0u8; 4];
                    reader.read_exact(&mut raw_pixel)?;
                    image_out.push(RGBA8 {
                        a: raw_pixel[i],
                        r: raw_pixel[i + 1],
                        g: raw_pixel[i + 2],
                        b: raw_pixel[i + 3],
                    });
                }
                Ok(image_out)
            }
            DataType::BlockCompression => match self.bc_type {
                BlockCompressionType::DXT1 => {
                    let mut image_out = vec![
                        RGBA8 {
                            r: 0,
                            g: 0,
                            b: 0,
                            a: 0
                        };
                        (self.dimensions.width * self.dimensions.height)
                            as usize
                    ];
                    let order_dimensions = ImageSize {
                        width: self.dimensions.width / 4,
                        height: self.dimensions.height / 4,
                    };
                    for i in 0..(order_dimensions.width * order_dimensions.height) {
                        let tile = decode_bc1_block(reader)?;
                        let tile_write_position = morton_order(i, order_dimensions);
                        for tile_y in 0..4 {
                            for tile_x in 0..4 {
                                let actual_pos_x = (tile_write_position.x * 4) + tile_x;
                                let actual_pos_y = (tile_write_position.y * 4) + tile_y;
                                image_out[((actual_pos_y * self.dimensions.width) + actual_pos_x)
                                    as usize] = tile[((tile_y * 4) + tile_x) as usize];
                            }
                        }
                    }
                    Ok(image_out)
                }
                BlockCompressionType::DXT5 => {
                    unimplemented!("DXT5 is not implemented yet");
                }
                BlockCompressionType::None => Err(TIDError::NoFourCC),
            },
        }
    }
}

impl DataType {
    fn import<R: Read>(reader: &mut R) -> Result<DataType> {
        Ok(match reader.read_to_u8()? {
            0x84 => DataType::BlockCompression,
            0x90 => DataType::RGBA,
            0x92 => DataType::ARGB,
            0x94 => DataType::BlockCompression,
            0x9C => DataType::BlockCompression,
            x => return Err(TIDError::UnknownDataType(x)),
        })
    }
}

impl BlockCompressionType {
    fn import<R: Read>(reader: &mut R) -> Result<BlockCompressionType> {
        let mut four_cc = vec![0u8; 4];
        reader.read_exact(&mut four_cc)?;
        match four_cc.as_slice() {
            [0u8, 0u8, 0u8, 0u8] => Ok(BlockCompressionType::None),
            [b'D', b'X', b'T', b'1'] => Ok(BlockCompressionType::DXT1),
            [b'D', b'X', b'T', b'5'] => Ok(BlockCompressionType::DXT5),
            _ => Err(TIDError::UnknownFourCC(four_cc)),
        }
    }
}

impl Display for DataType {
    fn fmt(&self, f: &mut Formatter) -> FMTResult {
        match *self {
            DataType::BlockCompression => write!(f, "Block Compression"),
            DataType::RGBA => write!(f, "RGBA"),
            DataType::ARGB => write!(f, "ARGB"),
        }
    }
}

impl ImageSize {
    fn import<R: Read>(reader: &mut R) -> Result<ImageSize> {
        Ok(ImageSize {
            width: reader.read_le_to_u32()?,
            height: reader.read_le_to_u32()?,
        })
    }
}

impl Display for ImageSize {
    fn fmt(&self, f: &mut Formatter) -> FMTResult {
        write!(f, "{}x{}", self.width, self.height)
    }
}
