use image::{DynamicImage, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

// ──────────────────────────────────────────────────────────────
// 1. LIQUIFY — push, pull, bulge, pinch, swirl
// ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquifyParams {
    #[serde(default = "default_liquify_mode")]
    pub mode: String,
    #[serde(default = "default_liquify_center_x")]
    pub center_x: f32,
    #[serde(default = "default_liquify_center_y")]
    pub center_y: f32,
    #[serde(default = "default_liquify_radius")]
    pub radius: f32,
    #[serde(default = "default_liquify_strength")]
    pub strength: f32,
}

fn default_liquify_mode() -> String { "bulge".to_string() }
fn default_liquify_center_x() -> f32 { 0.5 }
fn default_liquify_center_y() -> f32 { 0.5 }
fn default_liquify_radius() -> f32 { 0.3 }
fn default_liquify_strength() -> f32 { 0.5 }

pub fn liquify(img: &DynamicImage, params: &LiquifyParams) -> DynamicImage {
    let src = img.to_rgba8();
    let (w, h) = src.dimensions();
    let mut dst = RgbaImage::new(w, h);

    let cx = (params.center_x * w as f32).clamp(0.0, w as f32 - 1.0);
    let cy = (params.center_y * h as f32).clamp(0.0, h as f32 - 1.0);
    let r_px = params.radius * w.min(h) as f32;
    let r2 = r_px * r_px;

    for y in 0..h {
        for x in 0..w {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist2 = dx * dx + dy * dy;

            if dist2 >= r2 {
                dst.put_pixel(x, y, *src.get_pixel(x, y));
                continue;
            }

            let dist = dist2.sqrt();
            let t = dist / r_px;
            let falloff = (1.0 - t * t).clamp(0.0, 1.0);

            let (sx, sy) = match params.mode.as_str() {
                "bulge" => {
                    let factor = 1.0 + params.strength * falloff;
                    (cx + dx * factor, cy + dy * factor)
                }
                "pinch" => {
                    let factor = 1.0 - params.strength * falloff;
                    (cx + dx * factor, cy + dy * factor)
                }
                "swirl" => {
                    let angle = params.strength * falloff * PI * 2.0;
                    let cos_a = angle.cos();
                    let sin_a = angle.sin();
                    (cx + dx * cos_a - dy * sin_a, cy + dx * sin_a + dy * cos_a)
                }
                "push" => {
                    let angle = dy.atan2(dx);
                    let push_dist = params.strength * r_px * falloff;
                    ((x as f32 + push_dist * angle.cos()).clamp(0.0, w as f32 - 1.0),
                     (y as f32 + push_dist * angle.sin()).clamp(0.0, h as f32 - 1.0))
                }
                _ => (x as f32, y as f32),
            };

            let px = sample_bilinear(&src, sx, sy, w, h);
            dst.put_pixel(x, y, px);
        }
    }

    DynamicImage::ImageRgba8(dst)
}

// ──────────────────────────────────────────────────────────────
// 2. DISPLACEMENT MAP — grayscale drives distortion
// ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplacementParams {
    pub map_image: String,
    #[serde(default = "default_displacement_scale")]
    pub scale_x: f32,
    #[serde(default = "default_displacement_scale")]
    pub scale_y: f32,
}

fn default_displacement_scale() -> f32 { 20.0 }

pub fn displacement_map(img: &DynamicImage, params: &DisplacementParams) -> DynamicImage {
    let map_img = match super::load_from_base64(&params.map_image) {
        Ok(m) => m.to_rgba8(),
        Err(_) => return img.clone(),
    };

    let src = img.to_rgba8();
    let (w, h) = src.dimensions();
    let mut dst = RgbaImage::new(w, h);

    let (mw, mh) = map_img.dimensions();

    for y in 0..h {
        for x in 0..w {
            let mx = ((x as f32 / w as f32) * mw as f32) as u32;
            let my = ((y as f32 / h as f32) * mh as f32) as u32;
            let map_px = map_img.get_pixel(mx.min(mw - 1), my.min(mh - 1));

            let lum = (0.299 * map_px[0] as f32 + 0.587 * map_px[1] as f32 + 0.114 * map_px[2] as f32) / 255.0;
            let offset_x = (lum - 0.5) * 2.0 * params.scale_x;
            let offset_y = (lum - 0.5) * 2.0 * params.scale_y;

            let sx = (x as f32 + offset_x).clamp(0.0, w as f32 - 1.0);
            let sy = (y as f32 + offset_y).clamp(0.0, h as f32 - 1.0);

            dst.put_pixel(x, y, sample_bilinear(&src, sx, sy, w, h));
        }
    }

    DynamicImage::ImageRgba8(dst)
}

// ──────────────────────────────────────────────────────────────
// 3. WARP — wave, pinch, spherical, fisheye
// ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WarpParams {
    #[serde(default = "default_warp_mode")]
    pub mode: String,
    #[serde(default = "default_warp_amplitude")]
    pub amplitude: f32,
    #[serde(default = "default_warp_frequency")]
    pub frequency: f32,
}

fn default_warp_mode() -> String { "wave".to_string() }
fn default_warp_amplitude() -> f32 { 10.0 }
fn default_warp_frequency() -> f32 { 0.05 }

pub fn warp(img: &DynamicImage, params: &WarpParams) -> DynamicImage {
    let src = img.to_rgba8();
    let (w, h) = src.dimensions();
    let mut dst = RgbaImage::new(w, h);

    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;
    let max_dim = w.max(h) as f32;

    for y in 0..h {
        for x in 0..w {
            let (sx, sy) = match params.mode.as_str() {
                "wave" => {
                    let offset_x = (y as f32 * params.frequency).sin() * params.amplitude;
                    let offset_y = (x as f32 * params.frequency).sin() * params.amplitude;
                    (x as f32 + offset_x, y as f32 + offset_y)
                }
                "ripple" => {
                    let dx = x as f32 - cx;
                    let dy = y as f32 - cy;
                    let dist = (dx * dx + dy * dy).sqrt();
                    let offset = (dist * params.frequency).sin() * params.amplitude;
                    let angle = dy.atan2(dx);
                    (x as f32 + offset * angle.cos(), y as f32 + offset * angle.sin())
                }
                "fisheye" => {
                    let dx = x as f32 - cx;
                    let dy = y as f32 - cy;
                    let dist = (dx * dx + dy * dy).sqrt();
                    let max_dist = (cx * cx + cy * cy).sqrt();
                    let norm = dist / max_dist;
                    let factor = (norm * params.amplitude * 0.1).tanh();
                    let new_dist = dist * (1.0 - factor);
                    let angle = dy.atan2(dx);
                    (cx + new_dist * angle.cos(), cy + new_dist * angle.sin())
                }
                "barrel" => {
                    let dx = x as f32 - cx;
                    let dy = y as f32 - cy;
                    let dist = (dx * dx + dy * dy).sqrt();
                    let max_dist = max_dim / 2.0;
                    let norm = (dist / max_dist).clamp(0.0, 1.0);
                    let factor = 1.0 + params.amplitude * 0.01 * norm * norm;
                    (cx + dx * factor, cy + dy * factor)
                }
                "pincushion" => {
                    let dx = x as f32 - cx;
                    let dy = y as f32 - cy;
                    let dist = (dx * dx + dy * dy).sqrt();
                    let max_dist = max_dim / 2.0;
                    let norm = (dist / max_dist).clamp(0.0, 1.0);
                    let factor = 1.0 / (1.0 + params.amplitude * 0.01 * norm * norm);
                    (cx + dx * factor, cy + dy * factor)
                }
                _ => (x as f32, y as f32),
            };

            let sx_clamped = sx.clamp(0.0, w as f32 - 1.0);
            let sy_clamped = sy.clamp(0.0, h as f32 - 1.0);
            dst.put_pixel(x, y, sample_bilinear(&src, sx_clamped, sy_clamped, w, h));
        }
    }

    DynamicImage::ImageRgba8(dst)
}

// ──────────────────────────────────────────────────────────────
// 4. CORNER PIN — perspective / quadrilateral warp
// ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CornerPinParams {
    pub top_left_x: f32,
    pub top_left_y: f32,
    pub top_right_x: f32,
    pub top_right_y: f32,
    pub bottom_right_x: f32,
    pub bottom_right_y: f32,
    pub bottom_left_x: f32,
    pub bottom_left_y: f32,
}

pub fn corner_pin(img: &DynamicImage, params: &CornerPinParams) -> DynamicImage {
    let src = img.to_rgba8();
    let (w, h) = src.dimensions();
    let mut dst = RgbaImage::new(w, h);

    let src_corners = [
        [0.0_f32, 0.0_f32],
        [w as f32 - 1.0, 0.0_f32],
        [w as f32 - 1.0, h as f32 - 1.0],
        [0.0_f32, h as f32 - 1.0],
    ];

    let dst_corners = [
        [params.top_left_x, params.top_left_y],
        [params.top_right_x, params.top_right_y],
        [params.bottom_right_x, params.bottom_right_y],
        [params.bottom_left_x, params.bottom_left_y],
    ];

    let h_matrix = compute_homography(&src_corners, &dst_corners);

    for y in 0..h {
        for x in 0..w {
            let (sx, sy) = apply_homography(h_matrix, x as f32, y as f32);
            if sx >= 0.0 && sx < w as f32 - 1.0 && sy >= 0.0 && sy < h as f32 - 1.0 {
                dst.put_pixel(x, y, sample_bilinear(&src, sx, sy, w, h));
            } else {
                dst.put_pixel(x, y, Rgba([0, 0, 0, 0]));
            }
        }
    }

    DynamicImage::ImageRgba8(dst)
}

// ──────────────────────────────────────────────────────────────
// 5. GLITCH — datamoshing, slice shift, RGB split
// ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlitchParams {
    #[serde(default = "default_glitch_mode")]
    pub mode: String,
    #[serde(default = "default_glitch_intensity")]
    pub intensity: f32,
    #[serde(default = "default_glitch_seed")]
    pub seed: u32,
}

fn default_glitch_mode() -> String { "slice_shift".to_string() }
fn default_glitch_intensity() -> f32 { 0.5 }
fn default_glitch_seed() -> u32 { 42 }

pub fn glitch(img: &DynamicImage, params: &GlitchParams) -> DynamicImage {
    let mut rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let mut rng = SimpleRng::new(params.seed);

    let intensity = params.intensity.clamp(0.0, 1.0);

    match params.mode.as_str() {
        "slice_shift" => {
            let slice_height = (h / (10 + (intensity * 30.0) as u32)).max(1);
            for slice_y in (0..h).step_by(slice_height as usize) {
                if rng.next() % 3 == 0 {
                    let shift = ((rng.next() % w) as f32 * intensity * 0.5) as i32;
                    let end_y = (slice_y + slice_height).min(h);
                    for y in slice_y..end_y {
                        let mut row_buf: Vec<Rgba<u8>> = Vec::with_capacity(w as usize);
                        for x in 0..w {
                            row_buf.push(*rgba.get_pixel(x, y));
                        }
                        for x in 0..w {
                            let src_x = ((x as i32 + shift).rem_euclid(w as i32)) as u32;
                            rgba.put_pixel(x, y, row_buf[src_x as usize]);
                        }
                    }
                }
            }
        }
        "rgb_split" => {
            let mut r_buf = RgbaImage::new(w, h);
            let mut g_buf = RgbaImage::new(w, h);
            let mut b_buf = RgbaImage::new(w, h);

            for y in 0..h {
                for x in 0..w {
                    let px = *rgba.get_pixel(x, y);
                    let r_shift = ((intensity * 20.0) as i32).max(1);
                    let b_shift = ((intensity * 15.0) as i32).max(1);

                    let rx = (x as i32 + r_shift).clamp(0, w as i32 - 1) as u32;
                    let bx = (x as i32 - b_shift).clamp(0, w as i32 - 1) as u32;

                    r_buf.put_pixel(rx, y, Rgba([px[0], 0, 0, px[3]]));
                    g_buf.put_pixel(x, y, Rgba([0, px[1], 0, px[3]]));
                    b_buf.put_pixel(bx, y, Rgba([0, 0, px[2], px[3]]));
                }
            }

            for y in 0..h {
                for x in 0..w {
                    let r = r_buf.get_pixel(x, y);
                    let g = g_buf.get_pixel(x, y);
                    let b = b_buf.get_pixel(x, y);
                    rgba.put_pixel(x, y, Rgba([
                        r[0].saturating_add(g[0]).saturating_add(b[0]).min(255),
                        r[1].saturating_add(g[1]).saturating_add(b[1]).min(255),
                        r[2].saturating_add(g[2]).saturating_add(b[2]).min(255),
                        (r[3].saturating_add(g[3]).saturating_add(b[3]) / 3).min(255),
                    ]));
                }
            }
        }
        "datamosh" => {
            let block_size = (8.0 + intensity * 40.0) as u32;
            for by in (0..h).step_by(block_size as usize) {
                for bx in (0..w).step_by(block_size as usize) {
                    if rng.next() % 5 == 0 {
                        let copy_from_x = (bx as i32 + ((rng.next() % w) as f32 * intensity * 0.3) as i32)
                            .clamp(0, w as i32 - block_size as i32) as u32;
                        let copy_from_y = (by as i32 + ((rng.next() % h) as f32 * intensity * 0.1) as i32)
                            .clamp(0, h as i32 - block_size as i32) as u32;

                        for dy in 0..block_size {
                            for dx in 0..block_size {
                                let sx = (copy_from_x + dx).min(w - 1);
                                let sy = (copy_from_y + dy).min(h - 1);
                                let dx_dst = (bx + dx).min(w - 1);
                                let dy_dst = (by + dy).min(h - 1);
                                rgba.put_pixel(dx_dst, dy_dst, *rgba.get_pixel(sx, sy));
                            }
                        }
                    }
                }
            }
        }
        "pixel_scramble" => {
            let mut positions: Vec<(u32, u32)> = (0..h).flat_map(|y| (0..w).map(move |x| (x, y))).collect();
            let scramble_count = ((w * h) as f32 * intensity * 0.1) as usize;
            for _ in 0..scramble_count {
                let i1 = (rng.next() % positions.len() as u32) as usize;
                let i2 = (rng.next() % positions.len() as u32) as usize;
                positions.swap(i1, i2);
            }
            let mut new_buf = RgbaImage::new(w, h);
            let flat_pixels: Vec<Rgba<u8>> = {
                let mut v = Vec::with_capacity((w * h) as usize);
                for y in 0..h {
                    for x in 0..w {
                        v.push(*rgba.get_pixel(x, y));
                    }
                }
                v
            };
            for (i, &(nx, ny)) in positions.iter().enumerate() {
                new_buf.put_pixel(nx, ny, flat_pixels[i]);
            }
            rgba = new_buf;
        }
        _ => {}
    }

    DynamicImage::ImageRgba8(rgba)
}

// ──────────────────────────────────────────────────────────────
// 6. CHROMATIC ABERRATION — color channel displacement
// ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChromaticAberrationParams {
    #[serde(default = "default_ca_shift")]
    pub shift: f32,
    #[serde(default = "default_ca_auto")]
    pub auto: bool,
}

fn default_ca_shift() -> f32 { 5.0 }
fn default_ca_auto() -> bool { false }

pub fn chromatic_aberration(img: &DynamicImage, params: &ChromaticAberrationParams) -> DynamicImage {
    let src = img.to_rgba8();
    let (w, h) = src.dimensions();
    let mut dst = RgbaImage::new(w, h);

    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;

    for y in 0..h {
        for x in 0..w {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            let max_dist = (cx * cx + cy * cy).sqrt();
            let norm = (dist / max_dist).clamp(0.0, 1.0);

            let shift_amount = if params.auto {
                params.shift * norm
            } else {
                params.shift
            };

            let angle = dy.atan2(dx);
            let shift_x = (shift_amount * angle.cos()) as i32;
            let shift_y = (shift_amount * angle.sin()) as i32;

            let rx = ((x as i32 + shift_x).clamp(0, w as i32 - 1)) as u32;
            let ry = ((y as i32 + shift_y).clamp(0, h as i32 - 1)) as u32;
            let bx = ((x as i32 - shift_x).clamp(0, w as i32 - 1)) as u32;
            let by = ((y as i32 - shift_y).clamp(0, h as i32 - 1)) as u32;

            let r_px = src.get_pixel(rx, ry);
            let g_px = src.get_pixel(x, y);
            let b_px = src.get_pixel(bx, by);

            dst.put_pixel(x, y, Rgba([r_px[0], g_px[1], b_px[2], g_px[3]]));
        }
    }

    DynamicImage::ImageRgba8(dst)
}

// ──────────────────────────────────────────────────────────────
// 7. PIXEL SORT — sort pixels along an axis by brightness
// ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PixelSortParams {
    #[serde(default = "default_sort_axis")]
    pub axis: String,
    #[serde(default = "default_sort_threshold_low")]
    pub threshold_low: f32,
    #[serde(default = "default_sort_threshold_high")]
    pub threshold_high: f32,
}

fn default_sort_axis() -> String { "horizontal".to_string() }
fn default_sort_threshold_low() -> f32 { 0.2 }
fn default_sort_threshold_high() -> f32 { 0.8 }

pub fn pixel_sort(img: &DynamicImage, params: &PixelSortParams) -> DynamicImage {
    let mut rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();

    let brightness = |px: &Rgba<u8>| {
        (0.299 * px[0] as f32 + 0.587 * px[1] as f32 + 0.114 * px[2] as f32) / 255.0
    };

    match params.axis.as_str() {
        "horizontal" => {
            for y in 0..h {
                let mut row: Vec<Rgba<u8>> = (0..w).map(|x| *rgba.get_pixel(x, y)).collect();
                let mut i = 0;
                while i < w {
                    let lum = brightness(&row[i as usize]);
                    if lum >= params.threshold_low && lum <= params.threshold_high {
                        let mut j = i + 1;
                        while j < w {
                            let lum_j = brightness(&row[j as usize]);
                            if lum_j < params.threshold_low || lum_j > params.threshold_high {
                                break;
                            }
                            j += 1;
                        }
                        row[i as usize..j as usize].sort_by(|a, b| {
                            brightness(a).partial_cmp(&brightness(b)).unwrap()
                        });
                        i = j;
                    } else {
                        i += 1;
                    }
                }
                for (x, px) in row.iter().enumerate() {
                    rgba.put_pixel(x as u32, y, *px);
                }
            }
        }
        "vertical" => {
            for x in 0..w {
                let mut col: Vec<Rgba<u8>> = (0..h).map(|y| *rgba.get_pixel(x, y)).collect();
                let mut i = 0;
                while i < h {
                    let lum = brightness(&col[i as usize]);
                    if lum >= params.threshold_low && lum <= params.threshold_high {
                        let mut j = i + 1;
                        while j < h {
                            let lum_j = brightness(&col[j as usize]);
                            if lum_j < params.threshold_low || lum_j > params.threshold_high {
                                break;
                            }
                            j += 1;
                        }
                        col[i as usize..j as usize].sort_by(|a, b| {
                            brightness(a).partial_cmp(&brightness(b)).unwrap()
                        });
                        i = j;
                    } else {
                        i += 1;
                    }
                }
                for (y, px) in col.iter().enumerate() {
                    rgba.put_pixel(x, y as u32, *px);
                }
            }
        }
        _ => {}
    }

    DynamicImage::ImageRgba8(rgba)
}

// ──────────────────────────────────────────────────────────────
// 8. SCANLINES — CRT / retro display effect
// ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanlinesParams {
    #[serde(default = "default_scanlines_spacing")]
    pub spacing: u32,
    #[serde(default = "default_scanlines_opacity")]
    pub opacity: f32,
    #[serde(default = "default_scanlines_thickness")]
    pub thickness: u32,
}

fn default_scanlines_spacing() -> u32 { 4 }
fn default_scanlines_opacity() -> f32 { 0.3 }
fn default_scanlines_thickness() -> u32 { 1 }

pub fn scanlines(img: &DynamicImage, params: &ScanlinesParams) -> DynamicImage {
    let mut rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let opacity = params.opacity.clamp(0.0, 1.0);

    for y in 0..h {
        if (y / params.spacing.max(1)) % (params.thickness.max(1) + 1) < params.thickness {
            for x in 0..w {
                let px = rgba.get_pixel_mut(x, y);
                px[0] = (px[0] as f32 * (1.0 - opacity)) as u8;
                px[1] = (px[1] as f32 * (1.0 - opacity)) as u8;
                px[2] = (px[2] as f32 * (1.0 - opacity)) as u8;
            }
        }
    }

    DynamicImage::ImageRgba8(rgba)
}

// ──────────────────────────────────────────────────────────────
// 9. HALFTONE — dots-based rendering
// ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HalftoneParams {
    #[serde(default = "default_halftone_dot_size")]
    pub dot_size: u32,
    #[serde(default = "default_halftone_angle")]
    pub angle: f32,
    #[serde(default = "default_halftone_mode")]
    pub mode: String,
}

fn default_halftone_dot_size() -> u32 { 8 }
fn default_halftone_angle() -> f32 { 45.0 }
fn default_halftone_mode() -> String { "color".to_string() }

pub fn halftone(img: &DynamicImage, params: &HalftoneParams) -> DynamicImage {
    let src = img.to_rgba8();
    let (w, h) = src.dimensions();
    let mut dst = RgbaImage::from_pixel(w, h, Rgba([255, 255, 255, 255]));

    let dot_size = params.dot_size.max(2);
    let half_dot = dot_size as f32 / 2.0;
    let angle_rad = params.angle.to_radians();
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();

    let mode_color = params.mode == "color";

    for y in 0..h {
        for x in 0..w {
            let cell_x = (x / dot_size) * dot_size;
            let cell_y = (y / dot_size) * dot_size;
            let local_x = x as f32 - cell_x as f32 - half_dot;
            let local_y = y as f32 - cell_y as f32 - half_dot;

            let rot_x = local_x * cos_a - local_y * sin_a;
            let rot_y = local_x * sin_a + local_y * cos_a;
            let dist = (rot_x * rot_x + rot_y * rot_y).sqrt();

            let px = src.get_pixel(cell_x + half_dot as u32, cell_y + half_dot as u32);
            let lum = if mode_color {
                (px[0] as f32 + px[1] as f32 + px[2] as f32) / (3.0 * 255.0)
            } else {
                (0.299 * px[0] as f32 + 0.587 * px[1] as f32 + 0.114 * px[2] as f32) / 255.0
            };

            let max_radius = half_dot * lum.sqrt();

            if dist <= max_radius {
                if mode_color {
                    dst.put_pixel(x, y, *px);
                } else {
                    let gray = (lum * 255.0) as u8;
                    dst.put_pixel(x, y, Rgba([gray, gray, gray, px[3]]));
                }
            }
        }
    }

    DynamicImage::ImageRgba8(dst)
}

// ──────────────────────────────────────────────────────────────
// 10. POSTERIZE — reduce color levels
// ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PosterizeParams {
    #[serde(default = "default_posterize_levels")]
    pub levels: u8,
}

fn default_posterize_levels() -> u8 { 4 }

pub fn posterize(img: &DynamicImage, params: &PosterizeParams) -> DynamicImage {
    let mut rgba = img.to_rgba8();
    let levels = params.levels.max(2).min(64);
    let step = 255.0 / (levels - 1) as f32;

    for pixel in rgba.pixels_mut() {
        pixel[0] = ((pixel[0] as f32 / step).round() * step) as u8;
        pixel[1] = ((pixel[1] as f32 / step).round() * step) as u8;
        pixel[2] = ((pixel[2] as f32 / step).round() * step) as u8;
    }

    DynamicImage::ImageRgba8(rgba)
}

// ──────────────────────────────────────────────────────────────
// 11. NOISE — perlin/simplex style grain
// ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoiseParams {
    #[serde(default = "default_noise_amount")]
    pub amount: f32,
    #[serde(default = "default_noise_type")]
    pub noise_type: String,
    #[serde(default = "default_noise_seed")]
    pub seed: u32,
}

fn default_noise_amount() -> f32 { 0.1 }
fn default_noise_type() -> String { "grain".to_string() }
fn default_noise_seed() -> u32 { 42 }

pub fn noise(img: &DynamicImage, params: &NoiseParams) -> DynamicImage {
    let mut rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let amount = params.amount.clamp(0.0, 1.0);
    let mut rng = SimpleRng::new(params.seed);

    match params.noise_type.as_str() {
        "grain" => {
            for pixel in rgba.pixels_mut() {
                let n = ((rng.next() % 256) as i32 - 128) as f32 * amount;
                pixel[0] = (pixel[0] as f32 + n).clamp(0.0, 255.0) as u8;
                pixel[1] = (pixel[1] as f32 + n).clamp(0.0, 255.0) as u8;
                pixel[2] = (pixel[2] as f32 + n).clamp(0.0, 255.0) as u8;
            }
        }
        "color" => {
            for pixel in rgba.pixels_mut() {
                let nr = ((rng.next() % 256) as i32 - 128) as f32 * amount;
                let ng = ((rng.next() % 256) as i32 - 128) as f32 * amount;
                let nb = ((rng.next() % 256) as i32 - 128) as f32 * amount;
                pixel[0] = (pixel[0] as f32 + nr).clamp(0.0, 255.0) as u8;
                pixel[1] = (pixel[1] as f32 + ng).clamp(0.0, 255.0) as u8;
                pixel[2] = (pixel[2] as f32 + nb).clamp(0.0, 255.0) as u8;
            }
        }
        "monochrome" => {
            for y in 0..h {
                for x in 0..w {
                    let n = ((rng.next() % 256) as i32 - 128) as f32 * amount;
                    let g = n as u8;
                    let px = rgba.get_pixel_mut(x, y);
                    px[0] = px[0].saturating_add(g);
                    px[1] = px[1].saturating_add(g);
                    px[2] = px[2].saturating_add(g);
                }
            }
        }
        _ => {}
    }

    DynamicImage::ImageRgba8(rgba)
}

// ──────────────────────────────────────────────────────────────
// 12. KALEIDOSCOPE — mirror and repeat sectors
// ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KaleidoscopeParams {
    #[serde(default = "default_kaleidoscope_segments")]
    pub segments: u32,
    #[serde(default = "default_kaleidoscope_offset")]
    pub offset: f32,
}

fn default_kaleidoscope_segments() -> u32 { 6 }
fn default_kaleidoscope_offset() -> f32 { 0.0 }

pub fn kaleidoscope(img: &DynamicImage, params: &KaleidoscopeParams) -> DynamicImage {
    let src = img.to_rgba8();
    let (w, h) = src.dimensions();
    let mut dst = RgbaImage::new(w, h);

    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;
    let segments = params.segments.max(2).min(32) as f32;
    let angle_per_segment = PI * 2.0 / segments;

    for y in 0..h {
        for x in 0..w {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            let angle = dy.atan2(dx) + params.offset;

            let sector = ((angle / angle_per_segment).floor() as i32).rem_euclid(segments as i32);
            let sector_angle = sector as f32 * angle_per_segment;
            let local_angle = angle - sector_angle;

            let mirror_angle = if local_angle > angle_per_segment / 2.0 {
                sector_angle + angle_per_segment - local_angle
            } else {
                sector_angle + local_angle
            };

            let sx = (cx + dist * mirror_angle.cos()).clamp(0.0, w as f32 - 1.0) as u32;
            let sy = (cy + dist * mirror_angle.sin()).clamp(0.0, h as f32 - 1.0) as u32;

            dst.put_pixel(x, y, *src.get_pixel(sx.min(w - 1), sy.min(h - 1)));
        }
    }

    DynamicImage::ImageRgba8(dst)
}

// ──────────────────────────────────────────────────────────────
// 13. EMBOSS — 3D relief effect
// ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbossParams {
    #[serde(default = "default_emboss_strength")]
    pub strength: f32,
}

fn default_emboss_strength() -> f32 { 1.0 }

pub fn emboss(img: &DynamicImage, _params: &EmbossParams) -> DynamicImage {
    let src = img.to_rgba8();
    let (w, h) = src.dimensions();
    let mut dst = RgbaImage::new(w, h);

    let kernel = [
        [-2_i16, -1, 0],
        [-1, 1, 1],
        [0, 1, 2],
    ];

    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let mut r = 128_i32;
            let mut g = 128_i32;
            let mut b = 128_i32;

            for ky in 0..3 {
                for kx in 0..3 {
                    let px = src.get_pixel(x + kx as u32 - 1, y + ky as u32 - 1);
                    let k = kernel[ky][kx] as i32;
                    r += px[0] as i32 * k;
                    g += px[1] as i32 * k;
                    b += px[2] as i32 * k;
                }
            }

            let px = src.get_pixel(x, y);
            dst.put_pixel(x, y, Rgba([
                r.clamp(0, 255) as u8,
                g.clamp(0, 255) as u8,
                b.clamp(0, 255) as u8,
                px[3],
            ]));
        }
    }

    DynamicImage::ImageRgba8(dst)
}

// ──────────────────────────────────────────────────────────────
// 14. EDGE DETECT — Sobel edge detection
// ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EdgeDetectParams {
    #[serde(default = "default_edge_threshold")]
    pub threshold: u8,
    #[serde(default = "default_edge_invert")]
    pub invert: bool,
}

fn default_edge_threshold() -> u8 { 30 }
fn default_edge_invert() -> bool { false }

pub fn edge_detect(img: &DynamicImage, params: &EdgeDetectParams) -> DynamicImage {
    let src = img.to_rgba8();
    let (w, h) = src.dimensions();
    let mut dst = RgbaImage::new(w, h);
    let threshold = params.threshold as i32;

    let gx = [
        [-1_i16, 0, 1],
        [-2, 0, 2],
        [-1, 0, 1],
    ];
    let gy = [
        [-1_i16, -2, -1],
        [0, 0, 0],
        [1, 2, 1],
    ];

    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let mut sx = 0_i32;
            let mut sy = 0_i32;

            for ky in 0..3 {
                for kx in 0..3 {
                    let px = src.get_pixel(x + kx as u32 - 1, y + ky as u32 - 1);
                    let gray = (0.299 * px[0] as f32 + 0.587 * px[1] as f32 + 0.114 * px[2] as f32) as i32;
                    sx += gray * gx[ky][kx] as i32;
                    sy += gray * gy[ky][kx] as i32;
                }
            }

            let magnitude = ((sx * sx + sy * sy) as f32).sqrt() as i32;
            let edge = if magnitude > threshold { 255 } else { 0 };
            let px = src.get_pixel(x, y);

            if params.invert {
                dst.put_pixel(x, y, Rgba([255 - edge as u8, 255 - edge as u8, 255 - edge as u8, px[3]]));
            } else {
                dst.put_pixel(x, y, Rgba([edge as u8, edge as u8, edge as u8, px[3]]));
            }
        }
    }

    DynamicImage::ImageRgba8(dst)
}

// ──────────────────────────────────────────────────────────────
// 15. SOLARIZE — invert pixels above threshold
// ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolarizeParams {
    #[serde(default = "default_solarize_threshold")]
    pub threshold: u8,
}

fn default_solarize_threshold() -> u8 { 128 }

pub fn solarize(img: &DynamicImage, params: &SolarizeParams) -> DynamicImage {
    let mut rgba = img.to_rgba8();

    for pixel in rgba.pixels_mut() {
        if pixel[0] > params.threshold {
            pixel[0] = 255 - pixel[0];
        }
        if pixel[1] > params.threshold {
            pixel[1] = 255 - pixel[1];
        }
        if pixel[2] > params.threshold {
            pixel[2] = 255 - pixel[2];
        }
    }

    DynamicImage::ImageRgba8(rgba)
}

// ──────────────────────────────────────────────────────────────
// Helper: Bilinear sampling
// ──────────────────────────────────────────────────────────────

fn sample_bilinear(src: &RgbaImage, x: f32, y: f32, w: u32, h: u32) -> Rgba<u8> {
    let x0 = x.floor().clamp(0.0, w as f32 - 1.0) as u32;
    let y0 = y.floor().clamp(0.0, h as f32 - 1.0) as u32;
    let x1 = (x0 + 1).min(w - 1);
    let y1 = (y0 + 1).min(h - 1);

    let fx = x - x0 as f32;
    let fy = y - y0 as f32;

    let p00 = src.get_pixel(x0, y0);
    let p10 = src.get_pixel(x1, y0);
    let p01 = src.get_pixel(x0, y1);
    let p11 = src.get_pixel(x1, y1);

    Rgba([
        bilerp(p00[0], p10[0], p01[0], p11[0], fx, fy),
        bilerp(p00[1], p10[1], p01[1], p11[1], fx, fy),
        bilerp(p00[2], p10[2], p01[2], p11[2], fx, fy),
        bilerp(p00[3], p10[3], p01[3], p11[3], fx, fy),
    ])
}

fn bilerp(c00: u8, c10: u8, c01: u8, c11: u8, fx: f32, fy: f32) -> u8 {
    let v0 = c00 as f32 * (1.0 - fx) + c10 as f32 * fx;
    let v1 = c01 as f32 * (1.0 - fx) + c11 as f32 * fx;
    (v0 * (1.0 - fy) + v1 * fy).clamp(0.0, 255.0) as u8
}

// ──────────────────────────────────────────────────────────────
// Helper: Simple deterministic RNG (no external rand crate)
// ──────────────────────────────────────────────────────────────

struct SimpleRng {
    state: u32,
}

impl SimpleRng {
    fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        (self.state / 65536) % 32768
    }
}

// ──────────────────────────────────────────────────────────────
// Helper: Homography for corner pin
// ──────────────────────────────────────────────────────────────

type Mat3x3 = [[f32; 3]; 3];

fn compute_homography(src: &[[f32; 2]; 4], dst: &[[f32; 2]; 4]) -> Mat3x3 {
    let mut a = [[0.0_f32; 8]; 8];
    let mut b = [0.0_f32; 8];

    for i in 0..4 {
        let (sx, sy) = (src[i][0], src[i][1]);
        let (dx, dy) = (dst[i][0], dst[i][1]);
        a[i * 2] = [sx, sy, 1.0, 0.0, 0.0, 0.0, -dx * sx, -dx * sy];
        a[i * 2 + 1] = [0.0, 0.0, 0.0, sx, sy, 1.0, -dy * sx, -dy * sy];
        b[i * 2] = dx;
        b[i * 2 + 1] = dy;
    }

    let h = solve_8x8(&a, &b);
    [
        [h[0], h[1], h[2]],
        [h[3], h[4], h[5]],
        [h[6], h[7], 1.0],
    ]
}

fn solve_8x8(a: &[[f32; 8]; 8], b: &[f32; 8]) -> [f32; 8] {
    let mut m = [[0.0_f32; 9]; 8];
    for i in 0..8 {
        for j in 0..8 {
            m[i][j] = a[i][j];
        }
        m[i][8] = b[i];
    }

    for col in 0..8 {
        let mut pivot = col;
        for row in (col + 1)..8 {
            if m[row][col].abs() > m[pivot][col].abs() {
                pivot = row;
            }
        }
        m.swap(col, pivot);

        let pivot_val = m[col][col];
        if pivot_val.abs() < 1e-10 {
            continue;
        }

        for j in col..9 {
            m[col][j] /= pivot_val;
        }

        for row in 0..8 {
            if row != col {
                let factor = m[row][col];
                for j in col..9 {
                    m[row][j] -= factor * m[col][j];
                }
            }
        }
    }

    let mut result = [0.0_f32; 8];
    for i in 0..8 {
        result[i] = m[i][8];
    }
    result
}

fn apply_homography(h: Mat3x3, x: f32, y: f32) -> (f32, f32) {
    let w = h[2][0] * x + h[2][1] * y + h[2][2];
    if w.abs() < 1e-10 {
        return (x, y);
    }
    let sx = (h[0][0] * x + h[0][1] * y + h[0][2]) / w;
    let sy = (h[1][0] * x + h[1][1] * y + h[1][2]) / w;
    (sx, sy)
}
