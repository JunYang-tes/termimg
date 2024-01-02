use self::termion::terminal_size;
use self::termion::terminal_size_pixels;
use image::DynamicImage;
use std::error::Error;
extern crate termion;
#[derive(Debug, Clone)]
pub struct TerminalSize {
    pub width: u16,
    pub height: u16,
    pub cols: u16,
    pub rows: u16,
}


pub type DisplayResult = Result<(), Box<dyn Error>>;

pub trait Graphic {
    fn name(&self) -> &'static str;
    fn size(&self) -> Option<TerminalSize> {
        let (width, height) = terminal_size_pixels().unwrap_or((0, 0));
        let (cols, rows) = terminal_size().unwrap_or((0, 0));
        if width == 0 || height == 0 || cols == 0 || rows == 0 {
            return None;
        } else {
            return Some(TerminalSize {
                width,
                height,
                cols,
                rows,
            });
        }
    }
    fn display(&self, img: &DynamicImage) -> DisplayResult;
    fn supported(&self) -> bool;
}
