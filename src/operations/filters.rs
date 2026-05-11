use image::{DynamicImage, Rgba, RgbaImage};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlurParams {
    pub sigma: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrightnessParams {
    pub value: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContrastParams {
    pub value: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaturationParams {
    pub value: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrayscaleParams {
    #[serde(default = "default_grayscale_mode")]
    pub mode: String,
}

fn default_grayscale_mode() -> String {
    "luma".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TintParams {
    pub color: String,
    #[serde(default = "default_tint_amount")]
    pub amount: f32,
}

fn default_tint_amount() -> f32 {
    0.5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShadowParams {
    pub offset_x: i32,
    pub offset_y: i32,
    pub blur_radius: u32,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VignetteParams {
    #[serde(default = "default_vignette_strength")]
    pub strength: f32,
    #[serde(default = "default_vignette_radius")]
    pub radius: f32,
    #[serde(default)]
    pub color: Option<String>,
}

fn default_vignette_strength() -> f32 {
    0.5
}

fn default_vignette_radius() -> f32 {
    1.0
}

pub fn blur(img: &DynamicImage, params: &BlurParams) -> DynamicImage {
    img.blur(params.sigma)
}

pub fn brightness(img: &DynamicImage, params: &BrightnessParams) -> DynamicImage {
    img.brighten(params.value as i32)
}

pub fn contrast(img: &DynamicImage, params: &ContrastParams) -> DynamicImage {
    img.adjust_contrast(params.value)
}

pub fn grayscale(img: &DynamicImage, _params: &GrayscaleParams) -> DynamicImage {
    img.grayscale()
}

pub fn tint(img: &DynamicImage, params: &TintParams) -> DynamicImage {
    let mut rgba = img.to_rgba8();
    let tint_color = super::hex_to_rgba(&params.color).unwrap_or(Rgba([128, 128, 128, 255]));
    let amount = params.amount.clamp(0.0, 1.0);

    rgba.par_chunks_exact_mut(4).for_each(|pixel| {
        pixel[0] = (pixel[0] as f32 * (1.0 - amount) + tint_color[0] as f32 * amount) as u8;
        pixel[1] = (pixel[1] as f32 * (1.0 - amount) + tint_color[1] as f32 * amount) as u8;
        pixel[2] = (pixel[2] as f32 * (1.0 - amount) + tint_color[2] as f32 * amount) as u8;
    });

    DynamicImage::ImageRgba8(rgba)
}

pub fn vignette(img: &DynamicImage, params: &VignetteParams) -> DynamicImage {
    let mut rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let cx = width as f32 / 2.0;
    let cy = height as f32 / 2.0;
    let max_dist = (cx * cx + cy * cy).sqrt() * params.radius;

    let vignette_color = params
        .color
        .as_ref()
        .and_then(|c| super::hex_to_rgba(c).ok())
        .unwrap_or(Rgba([0, 0, 0, 255]));

    let w = width as usize;
    rgba.par_chunks_exact_mut(4).enumerate().for_each(|(idx, pixel)| {
        let x = (idx % w) as f32;
        let y = (idx / w) as f32;
        let dx = x - cx;
        let dy = y - cy;
        let dist = (dx * dx + dy * dy).sqrt();
        let factor = (1.0 - (dist / max_dist).clamp(0.0, 1.0) * params.strength).clamp(0.0, 1.0);

        pixel[0] = (pixel[0] as f32 * factor + vignette_color[0] as f32 * (1.0 - factor)) as u8;
        pixel[1] = (pixel[1] as f32 * factor + vignette_color[1] as f32 * (1.0 - factor)) as u8;
        pixel[2] = (pixel[2] as f32 * factor + vignette_color[2] as f32 * (1.0 - factor)) as u8;
    });

    DynamicImage::ImageRgba8(rgba)
}

pub fn drop_shadow(img: &DynamicImage, params: &ShadowParams) -> DynamicImage {
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let _shadow_color = super::hex_to_rgba(&params.color).unwrap_or(Rgba([0, 0, 0, 128]));

    let new_width = width + params.offset_x.abs() as u32 + params.blur_radius * 2;
    let new_height = height + params.offset_y.abs() as u32 + params.blur_radius * 2;

    let mut result = RgbaImage::from_pixel(new_width, new_height, Rgba([0, 0, 0, 0]));

    let content_x = if params.offset_x < 0 {
        params.blur_radius + params.offset_x.abs() as u32
    } else {
        params.blur_radius
    };
    let content_y = if params.offset_y < 0 {
        params.blur_radius + params.offset_y.abs() as u32
    } else {
        params.blur_radius
    };

    for y in 0..height {
        for x in 0..width {
            let px = rgba.get_pixel(x, y);
            if px[3] > 0 {
                let sx = content_x + x;
                let sy = content_y + y;
                if sx < new_width && sy < new_height {
                    let alpha = px[3] as f32 / 255.0;
                    let existing = result.get_pixel(sx, sy);
                    result.put_pixel(sx, sy, Rgba([
                        ((px[0] as f32 * alpha + existing[0] as f32 * (1.0 - alpha)).min(255.0)) as u8,
                        ((px[1] as f32 * alpha + existing[1] as f32 * (1.0 - alpha)).min(255.0)) as u8,
                        ((px[2] as f32 * alpha + existing[2] as f32 * (1.0 - alpha)).min(255.0)) as u8,
                        ((alpha * 255.0 + existing[3] as f32 * (1.0 - alpha)).min(255.0)) as u8,
                    ]));
                }
            }
        }
    }

    DynamicImage::ImageRgba8(result)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuotoneParams {
    pub shadow_color: String,
    pub highlight_color: String,
}

pub fn duotone(img: &DynamicImage, params: &DuotoneParams) -> DynamicImage {
    let mut rgba = img.to_rgba8();
    let shadow = super::hex_to_rgba(&params.shadow_color).unwrap_or(Rgba([0, 0, 0, 255]));
    let highlight = super::hex_to_rgba(&params.highlight_color).unwrap_or(Rgba([255, 255, 255, 255]));

    rgba.par_chunks_exact_mut(4).for_each(|pixel| {
        let luminance = (0.299 * pixel[0] as f32 + 0.587 * pixel[1] as f32 + 0.114 * pixel[2] as f32) / 255.0;
        let t = luminance.clamp(0.0, 1.0);

        pixel[0] = (shadow[0] as f32 * (1.0 - t) + highlight[0] as f32 * t) as u8;
        pixel[1] = (shadow[1] as f32 * (1.0 - t) + highlight[1] as f32 * t) as u8;
        pixel[2] = (shadow[2] as f32 * (1.0 - t) + highlight[2] as f32 * t) as u8;
    });

    DynamicImage::ImageRgba8(rgba)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{GenericImageView, RgbaImage};

    fn make_red_img(w: u32, h: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(RgbaImage::from_pixel(w, h, Rgba([255, 0, 0, 255])))
    }

    fn make_black_img(w: u32, h: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(RgbaImage::from_pixel(w, h, Rgba([0, 0, 0, 255])))
    }

    fn make_white_img(w: u32, h: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(RgbaImage::from_pixel(w, h, Rgba([255, 255, 255, 255])))
    }

    #[test]
    fn tint_red_full() {
        let img = make_red_img(10, 10);
        let tinted = tint(&img, &TintParams { color: "#0000FF".to_string(), amount: 1.0 });
        let rgba = tinted.to_rgba8();
        let px = rgba.get_pixel(0, 0);
        assert_eq!(px.0, [0, 0, 255, 255]);
    }

    #[test]
    fn tint_red_half() {
        let img = make_red_img(10, 10);
        let tinted = tint(&img, &TintParams { color: "#0000FF".to_string(), amount: 0.5 });
        let rgba = tinted.to_rgba8();
        let px = rgba.get_pixel(0, 0);
        assert_eq!(px.0, [127, 0, 127, 255]);
    }

    #[test]
    fn duotone_black_shadow() {
        let img = make_black_img(4, 4);
        let result = duotone(&img, &DuotoneParams {
            shadow_color: "#000000".to_string(),
            highlight_color: "#FFFFFF".to_string(),
        });
        let rgba = result.to_rgba8();
        let px = rgba.get_pixel(0, 0);
        assert_eq!(px.0, [0, 0, 0, 255]);
    }

    #[test]
    fn duotone_white_highlight() {
        let img = make_white_img(4, 4);
        let result = duotone(&img, &DuotoneParams {
            shadow_color: "#000000".to_string(),
            highlight_color: "#FFFFFF".to_string(),
        });
        let rgba = result.to_rgba8();
        let px = rgba.get_pixel(0, 0);
        assert_eq!(px.0, [255, 255, 255, 255]);
    }

    #[test]
    fn vignette_center_bright() {
        let img = make_white_img(50, 50);
        let result = vignette(&img, &VignetteParams {
            strength: 1.0, radius: 1.0, color: None,
        });
        let rgba = result.to_rgba8();
        let center = rgba.get_pixel(25, 25);
        assert!(center.0[0] > 200, "center should be mostly bright");
    }

    #[test]
    fn blur_returns_image() {
        let img = make_red_img(10, 10);
        let result = blur(&img, &BlurParams { sigma: 2.0 });
        assert_eq!(result.dimensions(), (10, 10));
    }
}
