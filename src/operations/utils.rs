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

#[cfg(test)]
mod tests {
    use super::*;
    use image::{GenericImageView, RgbaImage};

    #[test]
    fn hex_to_rgba_6_char() {
        let c = hex_to_rgba("#FF5733").unwrap();
        assert_eq!(c, Rgba([255, 87, 51, 255]));
    }

    #[test]
    fn hex_to_rgba_8_char() {
        let c = hex_to_rgba("#FF5733AA").unwrap();
        assert_eq!(c, Rgba([255, 87, 51, 170]));
    }

    #[test]
    fn hex_to_rgba_no_hash() {
        let c = hex_to_rgba("FF5733").unwrap();
        assert_eq!(c, Rgba([255, 87, 51, 255]));
    }

    #[test]
    fn hex_to_rgba_invalid() {
        assert!(hex_to_rgba("GGG").is_err());
    }

    #[test]
    fn base64_roundtrip() {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(10, 10, Rgba([255, 0, 0, 255])));
        let b64 = save_to_base64(&img, ImageFormat::Png).unwrap();
        let decoded = load_from_base64(&b64).unwrap();
        assert_eq!(decoded.dimensions(), (10, 10));
    }

    #[test]
    fn ensure_rgba_already_rgba() {
        let img = DynamicImage::ImageRgba8(RgbaImage::new(5, 5));
        let out = ensure_rgba(img);
        assert!(matches!(out, DynamicImage::ImageRgba8(_)));
    }
}
