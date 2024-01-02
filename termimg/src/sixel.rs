extern crate sixel_rs;
use std::io::stdout;
use std::path::Path;

use self::sixel_rs::sys::PixelFormat;
use image::{DynamicImage, GenericImageView};

use crate::graphic::{DisplayResult, Graphic};
use crate::term::{self, write};
use crate::utils::{get_image, has_alpha, prepare_img};
//https://vt100.net/docs/vt3xx-gp/chapter14.html
// xterm -ti vt340
#[derive(thiserror::Error, Debug)]
enum SixelError {
    #[error("Failed to initialize encoder")]
    FailedToGetEncoder,
    #[error("Faield to encode")]
    FailedToEncode,
}

pub struct Sixel;
impl Graphic for Sixel {
    fn name(&self) -> &'static str {
        "sixel"
    }

    fn display(&self, img: &DynamicImage) -> DisplayResult {
        let terminal_size = self.size();
        let encoder = sixel_rs::encoder::Encoder::new();
        let tmp_file = Path::new("/tmp/sixel.output");
        let _ = std::fs::remove_file(tmp_file);
        match encoder {
            Err(err) => Err(Box::new(SixelError::FailedToGetEncoder)),
            Ok(encoder) => {
                let (w, h) = img.dimensions();
                encoder.set_output(&tmp_file);
                encoder
                    .encode_bytes(
                        sixel_rs::encoder::QuickFrameBuilder::new()
                            .width(w as usize)
                            .height(h as usize)
                            .format(if has_alpha(&img) {
                                PixelFormat::RGBA8888
                            } else {
                                PixelFormat::RGB888
                            })
                            .pixels(img.to_bytes()),
                    )
                    .map_err(|_| -> DisplayResult { Err(Box::new(SixelError::FailedToEncode)) });
                let mut file = std::fs::File::open(tmp_file).unwrap();
                std::io::copy(&mut file, &mut stdout().lock());
                Ok(())
            }
        }
    }

    fn supported(&self) -> bool {
        // https://vt100.net/docs/vt510-rm/DA1.html
        let resp = write(&[0x1b, b'[', b'c']);
        resp.split(';')
            .into_iter()
            .any(|item| item.trim() == "4" || item.trim() == "4c")
    }
}
