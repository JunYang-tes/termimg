use image::{DynamicImage, GenericImage};

use crate::graphic::TerminalSize;

#[derive(Debug)]
pub enum LoadImageError {
    SvgError(String),
    ImageError(image::ImageError),
}

impl std::fmt::Display for LoadImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            LoadImageError::SvgError(msg) => write!(f, "{}", msg),
            LoadImageError::ImageError(err) => err.fmt(f),
        }
    }
}
impl std::error::Error for LoadImageError {}
impl From<image::ImageError> for LoadImageError {
    fn from(e: image::ImageError) -> Self {
        LoadImageError::ImageError(e)
    }
}

pub fn get_image(path: &String) -> std::result::Result<DynamicImage, LoadImageError> {
    if path.ends_with(".svg") {
        let svg_root = usvg::Tree::from_file(path, &usvg::Options::default());
        if let Err(_) = svg_root {
            return Err(LoadImageError::SvgError("Failed to load svg".to_string()));
        }
        let svg_root = svg_root.unwrap();
        let svg_image = resvg::render(&svg_root, usvg::FitTo::Width(1000), None);
        if let Some(svg_image) = svg_image {
            let mut dyn_img = DynamicImage::new_rgba8(svg_image.width(), svg_image.height());
            let data = svg_image.data();
            for x in 0..svg_image.width() {
                for y in 0..svg_image.height() {
                    let ind: usize = ((y * svg_image.width() + x) * 4) as usize;
                    let r = data[ind];
                    let g = data[ind + 1];
                    let b = data[ind + 2];
                    let a = data[ind + 3];

                    dyn_img.put_pixel(x, y, image::Rgba([r, g, b, a]))
                }
            }
            return Ok(dyn_img);
        }
    }
    Ok(image::open(path)?)
}
pub fn convert_to_rgb_rgba(img: DynamicImage) -> DynamicImage {
    match img.color() {
        image::ColorType::Rgb8 => img,
        image::ColorType::Rgba8 => img,
        _ => {
            if has_alpha(&img) {
                DynamicImage::ImageRgba8(img.to_rgba())
                // img.to_rgba()
                //     .map(|img| DynamicImage::ImageRgba8(img.clone()))
            } else {
                DynamicImage::ImageRgb8(img.to_rgb())
            }
        }
    }
}

pub fn prepare_img(path: &String, size: &Option<TerminalSize>) -> Result<DynamicImage, String> {
    let size = size.as_ref().unwrap_or(&TerminalSize {
        width: 0,
        height: 0,
        cols: 0,
        rows: 0,
    });
    let width = size.width;
    let height = size.height;
    get_image(&path)
        .map_err(|e| e.to_string())
        .map(convert_to_rgb_rgba)
        .and_then(|img| {
            if width == 0 || height == 0 {
                return Ok(img);
            }
            let (w, _) = image::GenericImageView::dimensions(&img);
            if w > (width as u32) {
                let scale = (w as f32) / (width as f32);
                let new_height = (scale * (height as f32)) as u32;
                Ok(img.resize(
                    width as u32,
                    new_height,
                    image::imageops::FilterType::Nearest,
                ))
            } else {
                Ok(img)
            }
        })
}
pub fn has_alpha(img: &DynamicImage) -> bool {
    use image::ColorType;
    let color = img.color();
    color == image::ColorType::La8
        || color == ColorType::Rgba8
        || color == ColorType::La16
        || color == ColorType::Bgra8
}
