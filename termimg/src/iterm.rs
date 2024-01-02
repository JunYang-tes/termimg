extern crate base64;
use image::GenericImageView;

use self::base64::Engine;
use std::fmt::Write;

use crate::graphic::Graphic;

pub struct Iterm;
impl Graphic for Iterm {
    fn name(&self) -> &'static str {
        "iterm"
    }

    fn display(&self, img: &image::DynamicImage) -> Result<(), Box<dyn std::error::Error>> {
        let mut content = vec![];
        let _ = img.write_to(&mut content,image::ImageOutputFormat::Png);
        let eng = base64::engine::general_purpose::STANDARD;
        let encoded = eng.encode(content);
        let size = encoded.len();
        let mut str = String::from("\x1b]1337;");
        let (w, h) = img.dimensions();
        write!(&mut str, "File=size={};width={}px;height={}px;inline=1:{}\x07", 
                       size,
                       w,
                       h,encoded)
            .map_err(|err| Box::new(err))?;
        println!("{}",str);
        Ok(())
    }

    fn supported(&self) -> bool {
        std::env::var("TERM_PROGRAM").unwrap_or("".to_owned()) == "iTerm.app"
    }
}
