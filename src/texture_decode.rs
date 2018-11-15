use super::ImageSize;
use crate::Result;
use ez_io::ReadE;
use rgb::RGBA8;
use std::cmp::min;
use std::io::Read;

#[derive(Clone, Copy)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

// ---------------- Z-order stuff ----------------

pub fn morton_order(pixel_index: u32, size: ImageSize) -> Position {
    let width_height_min = min(size.width, size.height);
    let nb_bits = f64::from(width_height_min).log2() as u32;
    if size.height < size.width {
        let j = pixel_index >> (2 * nb_bits) << (2 * nb_bits)
            | (decode_morton_2y(pixel_index) & (width_height_min - 1)) << nb_bits
            | (decode_morton_2x(pixel_index) & (width_height_min - 1));
        Position {
            x: j / size.height,
            y: j % size.height,
        }
    } else {
        let j = pixel_index >> (2 * nb_bits) << (2 * nb_bits)
            | (decode_morton_2x(pixel_index) & (width_height_min - 1)) << nb_bits
            | (decode_morton_2y(pixel_index) & (width_height_min - 1));
        Position {
            x: j % size.width,
            y: j / size.width,
        }
    }
}

fn decode_morton_2x(code: u32) -> u32 {
    compact_1_by_1(code)
}

fn decode_morton_2y(code: u32) -> u32 {
    compact_1_by_1(code >> 1)
}

fn compact_1_by_1(input: u32) -> u32 {
    let mut x = input;
    x &= 0x5555_5555;
    x = (x ^ (x >> 1)) & 0x3333_3333;
    x = (x ^ (x >> 2)) & 0x0f0f_0f0f;
    x = (x ^ (x >> 4)) & 0x00ff_00ff;
    x = (x ^ (x >> 8)) & 0x0000_ffff;
    x
}

// ---------------- BC1 Stuff ----------------
// Almost copy pasted from https://github.com/ifeherva/bcndecode/blob/master/src/decode.rs

pub fn decode_bc1_block<R: Read>(reader: &mut R) -> Result<[RGBA8; 16]> {
    let mut out = [RGBA8 {
        r: 0,
        b: 0,
        g: 0,
        a: 0,
    }; 16];
    let mut palette = [RGBA8 {
        r: 0,
        b: 0,
        g: 0,
        a: 0,
    }; 4];
    let c0 = reader.read_le_to_u16()?;
    let c1 = reader.read_le_to_u16()?;
    let lut = reader.read_le_to_u32()?;
    palette[0] = decode_565(c0);
    let r0 = u16::from(palette[0].r);
    let g0 = u16::from(palette[0].g);
    let b0 = u16::from(palette[0].b);
    palette[1] = decode_565(c1);
    let r1 = u16::from(palette[1].r);
    let g1 = u16::from(palette[1].g);
    let b1 = u16::from(palette[1].b);
    if c0 > c1 {
        palette[2].r = ((2 * r0 + r1) / 3) as u8;
        palette[2].g = ((2 * g0 + g1) / 3) as u8;
        palette[2].b = ((2 * b0 + b1) / 3) as u8;
        palette[2].a = 0xff;
        palette[3].r = ((r0 + 2 * r1) / 3) as u8;
        palette[3].g = ((g0 + 2 * g1) / 3) as u8;
        palette[3].b = ((b0 + 2 * b1) / 3) as u8;
        palette[3].a = 0xff;
    } else {
        palette[2].r = ((r0 + r1) / 2) as u8;
        palette[2].g = ((g0 + g1) / 2) as u8;
        palette[2].b = ((b0 + b1) / 2) as u8;
        palette[2].a = 0xff;
        palette[3].r = 0;
        palette[3].g = 0;
        palette[3].b = 0;
        palette[3].a = 0;
    }
    for n in 0..16 {
        let cw = (3 & (lut >> (2 * n))) as usize;
        out[n] = palette[cw];
    }
    Ok(out)
}

fn decode_565(color: u16) -> RGBA8 {
    let mut r: isize = ((color & 0xf800) >> 8) as isize;
    r |= r >> 5;

    let mut g: isize = ((color & 0x7e0) >> 3) as isize;
    g |= g >> 6;

    let mut b: isize = ((color & 0x1f) << 3) as isize;
    b |= b >> 5;

    RGBA8 {
        r: r as u8,
        g: g as u8,
        b: b as u8,
        a: 0xff,
    }
}
