extern crate docopt;
extern crate image;
extern crate infer;
extern crate resvg;
extern crate usvg;
#[macro_use]
extern crate serde_derive;
extern crate seek_bufread;
extern crate terminal_size;
extern crate imgcat;

use std::{
    io::{BufReader, Cursor, Read},
    path::Path,
};

use docopt::Docopt;
use image::*;
use infer::Type;
use terminal_size::{terminal_size, Height, Width};

mod apc;
mod graphic;
mod iterm;
mod kitty;
mod mosaic;
mod sixel;
mod term;
mod utils;
use graphic::Graphic;
use utils::prepare_img;

const USAGE: &'static str = "
    termimg : display image from <file> in an terminal

    Usage:
      termimg <file> [--protocol <protocol>]
      termimg --stdio
      termimg --list-protocol

    Options:
      --protocol <protocol>         One of kitty,mosaic,auto, [Default:auto]
      --list-protocol               Show protocols
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_protocol: Option<String>,
    flag_list_protocol: Option<bool>,
    arg_file: Option<String>,
}

fn main() {
    let viewers: Vec<Box<dyn Graphic>> = vec![
        Box::new(crate::kitty::Kitty {}),
        Box::new(crate::iterm::Iterm {}),
        Box::new(crate::sixel::Sixel {}),
        Box::new(crate::mosaic::Mosaic {}),
    ];
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());
    if args.flag_list_protocol.is_some() {
        for g in viewers.iter() {
            println!(
                "{} is {} supported",
                g.name(),
                if !g.supported() { "NOT" } else { "" }
            )
        }
        return;
    }
    let viewer = get_viewer(&viewers, args.flag_protocol.unwrap_or("auto".to_owned()))
        .expect("No viewer specified");
    let img = if args.arg_file.is_some() {
        let path = args.arg_file.unwrap();
        prepare_img(&path, &viewer.size()).unwrap()
    } else {
        read_img_from_stdio().unwrap()
    };
    viewer.display(&img);
}
fn read_img_from_stdio() -> Result<DynamicImage, String> {
    let mut buffer = vec![];
    std::io::stdin()
        .read_to_end(&mut buffer)
        .map_err(|err| err.to_string())?;
    let buffer = buffer.as_bytes();
    let reader = seek_bufread::BufReader::new(Cursor::new(&buffer));

    // let buffer = buffer.as_bytes();
    if let Some(kind) = infer::get(&buffer) {
        if let Some(fmt) = get_image_format(&kind) {
            let img = image::load(reader, fmt).map_err(|err| err.to_string())?;
            Ok(img)
        } else {
            Err("Failed to decode Image".into())
        }
    } else {
        Err("Unknow date format from stdin".into())
    }
}
fn get_image_format(typ: &Type) -> Option<ImageFormat> {
    match typ.mime_type() {
        "image/jpeg" => Some(ImageFormat::Jpeg),
        "image/png" => Some(ImageFormat::Png),
        "image/gif" => Some(ImageFormat::Gif),
        "image/webp" => Some(ImageFormat::WebP),
        "image/tiff" => Some(ImageFormat::Tiff),
        "image/bmp" => Some(ImageFormat::Bmp),
        _ => None,
    }
}
fn get_viewer(viewers: &Vec<Box<dyn Graphic>>, name: String) -> Option<&Box<dyn Graphic>> {
    if name == "auto" {
        for v in viewers.iter() {
            if v.supported() {
                return Some(v);
            }
        }
    } else {
        for v in viewers.iter() {
            if v.name() == name {
                return Some(v);
            }
        }
    }
    None
}
