extern crate clap;
extern crate png;
extern crate tid;
extern crate rgb;

use clap::{App, Arg};
use std::io::{BufReader, BufWriter};
use std::fs::File;
use tid::TID;
use png::{HasParameters, Encoder};
use rgb::ComponentBytes;

fn main() {
    let matches = App::new("Test TID Export")
        .version("0.1")
        .author("Marime Gui")
        .about("Test TID converter")
        .arg(Arg::with_name("INPUT").index(1).required(true))
        .arg(Arg::with_name("OUTPUT").index(2).required(true))
        .get_matches();
    let input = matches.value_of("INPUT").expect("No INPUT");
    let reader = &mut BufReader::new(File::open(input).expect("No such Input file"));
    let tid = TID::import(reader).expect("Error while importing TID");
    println!("Imported Image");
    let image = tid.convert();
    println!("Converted image");
    let w = &mut BufWriter::new(
        File::create(
            matches
                .value_of("OUTPUT")
                .expect("Cannot create output file"),
        ).unwrap(),
    );
    let mut encoder = Encoder::new(w, tid.dimensions.width, tid.dimensions.height);
    encoder.set(png::ColorType::RGBA).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(image.as_bytes()).unwrap();
    println!("Wrote image successfully");
}