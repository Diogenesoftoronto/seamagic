use image::{DynamicImage, imageops};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RotateParams {
    pub degrees: f32,
    #[serde(default)]
    pub background_color: Option<String>,
}

pub fn rotate(img: &DynamicImage, params: &RotateParams) -> DynamicImage {
    let angle_rad = params.degrees.to_radians();
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();

    let rgba = if let Some(hex) = &params.background_color {
        super::hex_to_rgba(hex).unwrap_or(image::Rgba([255, 255, 255, 255]))
    } else {
        image::Rgba([255, 255, 255, 255])
    };

    let rgba_img = match img {
        DynamicImage::ImageRgba8(i) => i.clone(),
        _ => img.to_rgba8(),
    };

    let (w, h) = rgba_img.dimensions();
    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;

    let nw = ((w as f32 * cos_a.abs()) + (h as f32 * sin_a.abs())).ceil().max(1.0) as u32;
    let nh = ((w as f32 * sin_a.abs()) + (h as f32 * cos_a.abs())).ceil().max(1.0) as u32;

    let mut result = image::RgbaImage::from_pixel(nw, nh, rgba);
    let ncx = nw as f32 / 2.0;
    let ncy = nh as f32 / 2.0;

    for y in 0..nh {
        for x in 0..nw {
            let dx = x as f32 - ncx;
            let dy = y as f32 - ncy;

            let src_x = (cx + dx * cos_a + dy * sin_a).round() as i32;
            let src_y = (cy - dx * sin_a + dy * cos_a).round() as i32;

            if src_x >= 0 && src_y >= 0 && (src_x as u32) < w && (src_y as u32) < h {
                result.put_pixel(x, y, *rgba_img.get_pixel(src_x as u32, src_y as u32));
            }
        }
    }

    DynamicImage::ImageRgba8(result)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlipParams {
    pub horizontal: bool,
    pub vertical: bool,
}

pub fn flip(img: &DynamicImage, params: &FlipParams) -> DynamicImage {
    let mut result = img.clone();
    if params.horizontal {
        result = imageops::flip_horizontal(&result.to_rgba8()).into();
    }
    if params.vertical {
        result = imageops::flip_vertical(&result.to_rgba8()).into();
    }
    result
}
