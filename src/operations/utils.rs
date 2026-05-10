use base64::Engine;
use image::{DynamicImage, ImageFormat, Rgba};
use std::io::Cursor;

pub fn load_from_base64(base64_str: &str) -> anyhow::Result<DynamicImage> {
    let bytes = base64::engine::general_purpose::STANDARD.decode(base64_str)?;
    let img = image::load_from_memory(&bytes)?;
    Ok(img)
}

pub fn save_to_base64(img: &DynamicImage, format: ImageFormat) -> anyhow::Result<String> {
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    img.write_to(&mut cursor, format)?;
    Ok(base64::engine::general_purpose::STANDARD.encode(buffer))
}

pub fn hex_to_rgba(hex: &str) -> anyhow::Result<Rgba<u8>> {
    let hex = hex.trim_start_matches('#');
    let (r, g, b, a) = match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16)?;
            let g = u8::from_str_radix(&hex[2..4], 16)?;
            let b = u8::from_str_radix(&hex[4..6], 16)?;
            (r, g, b, 255)
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16)?;
            let g = u8::from_str_radix(&hex[2..4], 16)?;
            let b = u8::from_str_radix(&hex[4..6], 16)?;
            let a = u8::from_str_radix(&hex[6..8], 16)?;
            (r, g, b, a)
        }
        _ => return Err(anyhow::anyhow!("Invalid hex color format")),
    };
    Ok(Rgba([r, g, b, a]))
}

pub fn ensure_rgba(img: DynamicImage) -> DynamicImage {
    match img {
        DynamicImage::ImageRgba8(_) => img,
        _ => DynamicImage::ImageRgba8(img.to_rgba8()),
    }
}
