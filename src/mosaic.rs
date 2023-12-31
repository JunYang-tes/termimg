use std::{cmp::min, fmt::Write};

extern crate termion;
use image::{imageops::FilterType, GenericImageView};
use termpix::print_image;

use self::termion::terminal_size;

use crate::{
    graphic::{self, TerminalSize},
    utils::get_image,
};
pub struct Mosaic;
impl graphic::Graphic for Mosaic {
    fn display(&self, img: &image::DynamicImage) -> Result<(), String> {
        let (imgw, imgh) = img.dimensions();
        let (w, h) = terminal_size()
            .map(|(w, h)| (w as u32, h as u32))
            .unwrap_or((imgw, imgh));
        if w != imgw || h != imgh {
            let (w, h) = fit_to_size(imgw, imgh, w, h, None, None);
            Ok(print_image(img, true, w, h, FilterType::Nearest))
        } else {
            Ok(print_image(img, true, imgw, imgh, FilterType::Nearest))
        }
    }

    fn supported(&self) -> bool {
        true
    }

    fn name(&self) -> &'static str {
        "mosaic"
    }
}
pub fn fit_to_size(
    orig_width: u32,
    orig_height: u32,
    terminal_width: u32,
    terminal_height: u32,
    max_width: Option<u32>,
    max_height: Option<u32>,
) -> (u32, u32) {
    let target_width = match max_width {
        Some(max_width) => min(max_width, terminal_width),
        None => terminal_width,
    };

    //2 pixels per terminal row
    let target_height = 2 * match max_height {
        Some(max_height) => min(max_height, terminal_height),
        None => terminal_height,
    };

    let calculated_width = scale_dimension(target_height, orig_width, orig_height);
    if calculated_width <= target_width {
        (calculated_width, target_height)
    } else {
        (
            target_width,
            scale_dimension(target_width, orig_height, orig_width),
        )
    }
}
fn scale_dimension(other: u32, orig_this: u32, orig_other: u32) -> u32 {
    (orig_this as f32 * other as f32 / orig_other as f32 + 0.5) as u32
}
