use image::{DynamicImage, Rgba, RgbaImage, GenericImageView};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaskParams {
    pub mask_type: String,
    #[serde(default)]
pub feather: bool,
    #[serde(default = "default_feather_radius")]
    pub feather_radius: u32,
}

fn default_feather_radius() -> u32 {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapeMaskParams {
    pub shape: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub feather: bool,
    #[serde(default = "default_feather_radius")]
    pub feather_radius: u32,
}

pub fn apply_mask(img: &DynamicImage, mask: &DynamicImage) -> DynamicImage {
    let mut rgba = img.to_rgba8();
    let mask_rgba = mask.to_rgba8();
    let (w, h) = rgba.dimensions();

    for y in 0..h {
        for x in 0..w {
            if x < mask_rgba.width() && y < mask_rgba.height() {
                let mask_pixel = mask_rgba.get_pixel(x, y);
                let luminance = ((0.299 * mask_pixel[0] as f32) + (0.587 * mask_pixel[1] as f32) + (0.114 * mask_pixel[2] as f32)) / 255.0;
                let pixel = rgba.get_pixel_mut(x, y);
                pixel[3] = ((pixel[3] as f32 * luminance).min(255.0)) as u8;
            }
        }
    }

    DynamicImage::ImageRgba8(rgba)
}

pub fn create_shape_mask(img: &DynamicImage, params: &ShapeMaskParams) -> DynamicImage {
    let (width, height) = img.dimensions();
    let mut mask = RgbaImage::from_pixel(width, height, Rgba([0, 0, 0, 0]));

    match params.shape.as_str() {
        "circle" => {
            let cx = params.x + (params.width as i32) / 2;
            let cy = params.y + (params.height as i32) / 2;
            let radius = (params.width.min(params.height) as f32 / 2.0) as i32;

            for y in 0..height as i32 {
                for x in 0..width as i32 {
                    let dx = x - cx;
                    let dy = y - cy;
                    if dx * dx + dy * dy <= radius * radius {
                        mask.put_pixel(x as u32, y as u32, Rgba([255, 255, 255, 255]));
                    }
                }
            }
        }
        "ellipse" => {
            let cx = params.x + (params.width as i32) / 2;
            let cy = params.y + (params.height as i32) / 2;
            let rx = params.width as f32 / 2.0;
            let ry = params.height as f32 / 2.0;

            for y in 0..height as i32 {
                for x in 0..width as i32 {
                    let dx = (x - cx) as f32;
                    let dy = (y - cy) as f32;
                    if (dx * dx) / (rx * rx) + (dy * dy) / (ry * ry) <= 1.0 {
                        mask.put_pixel(x as u32, y as u32, Rgba([255, 255, 255, 255]));
                    }
                }
            }
        }
        "rounded_rect" => {
            let rx = 20;
            let x_start = params.x.max(0) as u32;
            let y_start = params.y.max(0) as u32;
            let x_end = (params.x + params.width as i32).min(width as i32) as u32;
            let y_end = (params.y + params.height as i32).min(height as i32) as u32;

            for y in y_start..y_end {
                for x in x_start..x_end {
                    let lx = x as i32 - params.x;
                    let ly = y as i32 - params.y;
                    let r = rx.min(params.width / 2).min(params.height / 2) as i32;

                    let in_rect = (lx >= r && lx < params.width as i32 - r)
                        || (ly >= r && ly < params.height as i32 - r);
                    let in_tl = lx < r && ly < r && (lx - r).pow(2) + (ly - r).pow(2) <= r * r;
                    let in_tr = lx >= params.width as i32 - r && ly < r
                        && (lx - (params.width as i32 - r)).pow(2) + (ly - r).pow(2) <= r * r;
                    let in_bl = lx < r && ly >= params.height as i32 - r
                        && (lx - r).pow(2) + (ly - (params.height as i32 - r)).pow(2) <= r * r;
                    let in_br = lx >= params.width as i32 - r && ly >= params.height as i32 - r
                        && (lx - (params.width as i32 - r)).pow(2) + (ly - (params.height as i32 - r)).pow(2) <= r * r;

                    if in_rect || in_tl || in_tr || in_bl || in_br {
                        mask.put_pixel(x, y, Rgba([255, 255, 255, 255]));
                    }
                }
            }
        }
        _ => {
            let x_start = params.x.max(0) as u32;
            let y_start = params.y.max(0) as u32;
            let x_end = (params.x + params.width as i32).min(width as i32) as u32;
            let y_end = (params.y + params.height as i32).min(height as i32) as u32;

            for y in y_start..y_end {
                for x in x_start..x_end {
                    mask.put_pixel(x, y, Rgba([255, 255, 255, 255]));
                }
            }
        }
    }

    if params.feather {
        apply_feather(&mut mask, params.feather_radius);
    }

    DynamicImage::ImageRgba8(mask)
}

fn apply_feather(mask: &mut RgbaImage, radius: u32) {
    let radius = radius.max(1) as i32;
    let (w, h) = mask.dimensions();
    let mut new_mask = mask.clone();

    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let mut sum = 0.0;
            let mut count = 0.0;

            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let dist = ((dx * dx + dy * dy) as f32).sqrt();
                    if dist > radius as f32 {
                        continue;
                    }

                    let px = x + dx;
                    let py = y + dy;
                    if px >= 0 && py >= 0 && px < w as i32 && py < h as i32 {
                        let weight = 1.0 - dist / radius as f32;
                        sum += mask.get_pixel(px as u32, py as u32)[0] as f32 * weight;
                        count += weight;
                    }
                }
            }

            if count > 0.0 {
                let avg = (sum / count) as u8;
                new_mask.put_pixel(x as u32, y as u32, Rgba([avg, avg, avg, 255]));
            }
        }
    }

    *mask = new_mask;
}
