use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayParams {
    pub base_image: String,
    pub overlay_image: String,
    pub x: i32,
    pub y: i32,
    #[serde(default)]
    pub opacity: Option<f32>,
    #[serde(default)]
    pub blend_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StitchParams {
    pub images: Vec<String>,
    pub direction: String,
    #[serde(default = "default_gap")]
    pub gap: u32,
    #[serde(default)]
    pub background_color: Option<String>,
}

fn default_gap() -> u32 {
    0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameParams {
    pub width: u32,
    pub height: u32,
    pub frame_width: u32,
    pub frame_color: String,
    pub inner_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollageParams {
    pub images: Vec<String>,
    pub layout: String,
    pub width: u32,
    pub height: u32,
    #[serde(default = "default_gap")]
    pub gap: u32,
}

pub fn overlay(base: &DynamicImage, overlay: &DynamicImage, params: &OverlayParams) -> DynamicImage {
    let mut result = base.to_rgba8();
    let overlay_rgba = overlay.to_rgba8();
    let opacity = params.opacity.unwrap_or(1.0).clamp(0.0, 1.0);

    for y in 0..overlay_rgba.height() {
        for x in 0..overlay_rgba.width() {
            let base_x = params.x + x as i32;
            let base_y = params.y + y as i32;

            if base_x >= 0 && base_y >= 0
                && base_x < result.width() as i32
                && base_y < result.height() as i32 {
                let ov = *overlay_rgba.get_pixel(x, y);
                if ov[3] == 0 { continue; }

                let bx = base_x as u32;
                let by = base_y as u32;
                let bg = *result.get_pixel(bx, by);

                let blend_mode = params.blend_mode.as_deref().unwrap_or("normal");

                let (r, g, b, a) = match blend_mode {
                    "multiply" => {
                        let alpha = (ov[3] as f32 / 255.0) * opacity;
                        (
                            ((bg[0] as f32 * ov[0] as f32) / 255.0) as u8,
                            ((bg[1] as f32 * ov[1] as f32) / 255.0) as u8,
                            ((bg[2] as f32 * ov[2] as f32) / 255.0) as u8,
                            (alpha * 255.0 + bg[3] as f32 * (1.0 - alpha)) as u8,
                        )
                    }
                    "screen" => {
                        let alpha = (ov[3] as f32 / 255.0) * opacity;
                        (
                            (255.0 - ((255.0 - bg[0] as f32) * (255.0 - ov[0] as f32) / 255.0)) as u8,
                            (255.0 - ((255.0 - bg[1] as f32) * (255.0 - ov[1] as f32) / 255.0)) as u8,
                            (255.0 - ((255.0 - bg[2] as f32) * (255.0 - ov[2] as f32) / 255.0)) as u8,
                            (alpha * 255.0 + bg[3] as f32 * (1.0 - alpha)) as u8,
                        )
                    }
                    "overlay" => {
                        let alpha = (ov[3] as f32 / 255.0) * opacity;
                        let blend = |bg: u8, ov: u8| {
                            if bg < 128 {
                                (2.0 * bg as f32 * ov as f32 / 255.0) as u8
                            } else {
                                (255.0 - 2.0 * (255.0 - bg as f32) * (255.0 - ov as f32) / 255.0) as u8
                            }
                        };
                        (
                            blend(bg[0], ov[0]),
                            blend(bg[1], ov[1]),
                            blend(bg[2], ov[2]),
                            (alpha * 255.0 + bg[3] as f32 * (1.0 - alpha)) as u8,
                        )
                    }
                    "darken" => {
                        let alpha = (ov[3] as f32 / 255.0) * opacity;
                        (
                            bg[0].min(ov[0]),
                            bg[1].min(ov[1]),
                            bg[2].min(ov[2]),
                            (alpha * 255.0 + bg[3] as f32 * (1.0 - alpha)) as u8,
                        )
                    }
                    "lighten" => {
                        let alpha = (ov[3] as f32 / 255.0) * opacity;
                        (
                            bg[0].max(ov[0]),
                            bg[1].max(ov[1]),
                            bg[2].max(ov[2]),
                            (alpha * 255.0 + bg[3] as f32 * (1.0 - alpha)) as u8,
                        )
                    }
                    _ => {
                        let alpha = (ov[3] as f32 / 255.0) * opacity;
                        (
                            (ov[0] as f32 * alpha + bg[0] as f32 * (1.0 - alpha)) as u8,
                            (ov[1] as f32 * alpha + bg[1] as f32 * (1.0 - alpha)) as u8,
                            (ov[2] as f32 * alpha + bg[2] as f32 * (1.0 - alpha)) as u8,
                            (alpha * 255.0 + bg[3] as f32 * (1.0 - alpha)) as u8,
                        )
                    }
                };

                result.put_pixel(bx, by, Rgba([r, g, b, a]));
            }
        }
    }

    DynamicImage::ImageRgba8(result)
}

pub fn create_frame(params: &FrameParams) -> DynamicImage {
    let mut img = RgbaImage::from_pixel(params.width, params.height, Rgba([0, 0, 0, 0]));
    let frame_color = super::hex_to_rgba(&params.frame_color).unwrap_or(Rgba([0, 0, 0, 255]));
    let inner_color = params.inner_color.as_ref()
        .and_then(|c| super::hex_to_rgba(c).ok())
        .unwrap_or(Rgba([0, 0, 0, 0]));

    for y in 0..params.height {
        for x in 0..params.width {
            let in_frame = x < params.frame_width
                || x >= params.width - params.frame_width
                || y < params.frame_width
                || y >= params.height - params.frame_width;

            if in_frame {
                img.put_pixel(x, y, frame_color);
            } else {
                img.put_pixel(x, y, inner_color);
            }
        }
    }

    DynamicImage::ImageRgba8(img)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComparisonParams {
    pub original_image: String,
    pub edited_image: String,
    #[serde(default = "default_comparison_orientation")]
    pub orientation: String,
    #[serde(default = "default_gap")]
    pub gap: u32,
    #[serde(default)]
    pub label_original: Option<String>,
    #[serde(default)]
    pub label_edited: Option<String>,
    #[serde(default = "default_label_size")]
    pub label_size: u32,
    #[serde(default = "default_label_color")]
    pub label_color: String,
    #[serde(default = "default_comparison_bg")]
    pub background_color: String,
}

fn default_comparison_orientation() -> String {
    "horizontal".to_string()
}

fn default_label_size() -> u32 {
    24
}

fn default_label_color() -> String {
    "#FFFFFF".to_string()
}

fn default_comparison_bg() -> String {
    "#1a1a2e".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepsGridParams {
    pub images: Vec<String>,
    pub labels: Option<Vec<String>>,
    pub width: u32,
    pub height: u32,
    #[serde(default = "default_gap")]
    pub gap: u32,
    #[serde(default = "default_label_size")]
    pub label_size: u32,
    #[serde(default = "default_label_color")]
    pub label_color: String,
    #[serde(default = "default_comparison_bg")]
    pub background_color: String,
    #[serde(default = "default_max_cols")]
    pub max_columns: u32,
}

fn default_max_cols() -> u32 {
    4
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffParams {
    pub original_image: String,
    pub edited_image: String,
    #[serde(default = "default_diff_threshold")]
    pub threshold: u32,
}

fn default_diff_threshold() -> u32 {
    30
}

pub fn create_comparison(original: &DynamicImage, edited: &DynamicImage, params: &ComparisonParams) -> DynamicImage {
    let gap = params.gap;
    let label_height = if params.label_original.is_some() || params.label_edited.is_some() { 40 } else { 0 };
    let bg = super::hex_to_rgba(&params.background_color).unwrap_or(Rgba([26, 26, 46, 255]));
    let label_color = super::hex_to_rgba(&params.label_color).unwrap_or(Rgba([255, 255, 255, 255]));

    match params.orientation.as_str() {
        "vertical" => {
            let max_w = original.width().max(edited.width());
            let total_h = original.height() + edited.height() + gap + label_height * 2;
            let mut canvas = RgbaImage::from_pixel(max_w, total_h, bg);

            let orig_resized = original.resize(max_w, original.height(), image::imageops::FilterType::Lanczos3);
            place_image(&mut canvas, &orig_resized, 0, 0);

            if let Some(ref label) = params.label_original {
                draw_label_simple(&mut canvas, label, max_w / 2, original.height() - 5, label_color, params.label_size);
            }

            let edit_y = original.height() + gap + label_height;
            let edit_resized = edited.resize(max_w, edited.height(), image::imageops::FilterType::Lanczos3);
            place_image(&mut canvas, &edit_resized, 0, edit_y);

            if let Some(ref label) = params.label_edited {
                draw_label_simple(&mut canvas, label, max_w / 2, edit_y + edited.height() - 5, label_color, params.label_size);
            }

            DynamicImage::ImageRgba8(canvas)
        }
        _ => {
            let max_h = original.height().max(edited.height());
            let total_w = original.width() + edited.width() + gap;
            let total_h = max_h + label_height;
            let mut canvas = RgbaImage::from_pixel(total_w, total_h, bg);

            let orig_resized = original.resize(original.width(), max_h, image::imageops::FilterType::Lanczos3);
            place_image(&mut canvas, &orig_resized, 0, 0);

            if let Some(ref label) = params.label_original {
                draw_label_simple(&mut canvas, label, original.width() / 2, 5, label_color, params.label_size);
            }

            let edit_x = original.width() + gap;
            let edit_resized = edited.resize(edited.width(), max_h, image::imageops::FilterType::Lanczos3);
            place_image(&mut canvas, &edit_resized, edit_x, 0);

            if let Some(ref label) = params.label_edited {
                draw_label_simple(&mut canvas, label, edit_x + edited.width() / 2, 5, label_color, params.label_size);
            }

            DynamicImage::ImageRgba8(canvas)
        }
    }
}

fn place_image(canvas: &mut RgbaImage, img: &DynamicImage, x: u32, y: u32) {
    let rgba = img.to_rgba8();
    for ry in 0..rgba.height() {
        for rx in 0..rgba.width() {
            let px = x + rx;
            let py = y + ry;
            if px < canvas.width() && py < canvas.height() {
                canvas.put_pixel(px, py, *rgba.get_pixel(rx, ry));
            }
        }
    }
}

fn draw_label_simple(canvas: &mut RgbaImage, text: &str, cx: u32, y: u32, color: Rgba<u8>, size: u32) {
    // Simple centered text rendering using block characters
    let text_upper = text.to_uppercase();
    let chars: Vec<char> = text_upper.chars().collect();
    let char_width = (size.max(8) as f32 * 0.6) as u32;
    let total_width = chars.len() as u32 * char_width;
    let start_x = if cx > total_width / 2 { cx - total_width / 2 } else { 0 };

    for (i, c) in chars.iter().enumerate() {
        let px = start_x + i as u32 * char_width;
        draw_char_simple(canvas, *c, px as i32, y as i32, color, size);
    }
}

fn draw_char_simple(canvas: &mut RgbaImage, c: char, x: i32, y: i32, color: Rgba<u8>, size: u32) {
    let scale = (size.max(8) as f32 / 16.0).max(0.5);
    use crate::operations::text::get_char_pattern;
    let pattern = get_char_pattern(c);

    for (row_idx, row) in pattern.iter().enumerate() {
        for (col_idx, &pixel) in row.iter().enumerate() {
            if pixel {
                let px = (x as f32 + col_idx as f32 * scale) as i32;
                let py = (y as f32 + row_idx as f32 * scale) as i32;

                if scale > 1.0 {
                    for dy in 0..scale.ceil() as i32 {
                        for dx in 0..scale.ceil() as i32 {
                            let fx = px + dx;
                            let fy = py + dy;
                            if fx >= 0 && fy >= 0 && fx < canvas.width() as i32 && fy < canvas.height() as i32 {
                                canvas.put_pixel(fx as u32, fy as u32, color);
                            }
                        }
                    }
                } else {
                    if px >= 0 && py >= 0 && px < canvas.width() as i32 && py < canvas.height() as i32 {
                        canvas.put_pixel(px as u32, py as u32, color);
                    }
                }
            }
        }
    }
}

pub fn create_steps_grid(images: &Vec<DynamicImage>, params: &StepsGridParams) -> DynamicImage {
    let gap = params.gap;
    let count = images.len().max(1);
    let cols = (count as u32).min(params.max_columns).max(1);
    let rows = ((count as u32) + cols - 1) / cols;

    let label_height = if params.labels.as_ref().map(|l| !l.is_empty()).unwrap_or(false) { 30 } else { 0 };
    let cell_w = (params.width - gap * (cols + 1)) / cols;
    let cell_h = (params.height - gap * (rows + 1) - label_height * rows) / rows;
    let bg = super::hex_to_rgba(&params.background_color).unwrap_or(Rgba([26, 26, 46, 255]));
    let label_color = super::hex_to_rgba(&params.label_color).unwrap_or(Rgba([255, 255, 255, 255]));

    let mut canvas = RgbaImage::from_pixel(params.width, params.height, bg);

    for (i, img) in images.iter().enumerate() {
        let col = (i as u32) % cols;
        let row = (i as u32) / cols;
        let x = gap + col * (cell_w + gap);
        let y = gap + row * (cell_h + gap + label_height);

        let resized = img.resize(cell_w, cell_h, image::imageops::FilterType::Lanczos3);
        let (rw, rh) = resized.dimensions();
        let offset_x = x + (cell_w - rw) / 2;
        let offset_y = y + (cell_h - rh) / 2;

        place_image(&mut canvas, &resized, offset_x, offset_y);

        // Draw border around each cell
        draw_cell_border(&mut canvas, x, y, cell_w, cell_h, Rgba([80, 80, 100, 255]));

        if let Some(ref labels) = params.labels {
            if i < labels.len() {
                let label_y = y + cell_h + 2;
                draw_label_simple(&mut canvas, &labels[i], x + cell_w / 2, label_y, label_color, params.label_size);
            }
        }
    }

    DynamicImage::ImageRgba8(canvas)
}

fn draw_cell_border(canvas: &mut RgbaImage, x: u32, y: u32, w: u32, h: u32, color: Rgba<u8>) {
    for dx in 0..w {
        if x + dx < canvas.width() {
            if y < canvas.height() { canvas.put_pixel(x + dx, y, color); }
            if y + h - 1 < canvas.height() { canvas.put_pixel(x + dx, y + h - 1, color); }
        }
    }
    for dy in 0..h {
        if y + dy < canvas.height() {
            if x < canvas.width() { canvas.put_pixel(x, y + dy, color); }
            if x + w - 1 < canvas.width() { canvas.put_pixel(x + w - 1, y + dy, color); }
        }
    }
}

pub fn create_diff(original: &DynamicImage, edited: &DynamicImage, params: &DiffParams) -> DynamicImage {
    let threshold = params.threshold;
    let orig_rgba = original.to_rgba8();
    let edit_rgba = edited.to_rgba8();

    let (w, h) = orig_rgba.dimensions();
    let mut result = RgbaImage::from_pixel(w, h, Rgba([0, 0, 0, 255]));

    for y in 0..h {
        for x in 0..w {
            let o = *orig_rgba.get_pixel(x, y);
            let e = if x < edit_rgba.width() && y < edit_rgba.height() {
                *edit_rgba.get_pixel(x, y)
            } else {
                Rgba([0, 0, 0, 0])
            };

            let diff_r = (o[0] as i32 - e[0] as i32).abs() as u32;
            let diff_g = (o[1] as i32 - e[1] as i32).abs() as u32;
            let diff_b = (o[2] as i32 - e[2] as i32).abs() as u32;
            let diff_a = (o[3] as i32 - e[3] as i32).abs() as u32;

            let is_diff = diff_r > threshold || diff_g > threshold || diff_b > threshold || diff_a > threshold;

            if is_diff {
                // Highlight differences in cyan with original mixed in
                let blend_factor = 0.3;
                let r = (o[0] as f32 * (1.0 - blend_factor) + 0.0 * blend_factor) as u8;
                let g = (o[1] as f32 * (1.0 - blend_factor) + 255.0 * blend_factor) as u8;
                let b = (o[2] as f32 * (1.0 - blend_factor) + 255.0 * blend_factor) as u8;
                result.put_pixel(x, y, Rgba([r, g, b, 255]));
            } else {
                // Dim unchanged pixels
                result.put_pixel(x, y, Rgba([
                    (o[0] as f32 * 0.3) as u8,
                    (o[1] as f32 * 0.3) as u8,
                    (o[2] as f32 * 0.3) as u8,
                    255,
                ]));
            }
        }
    }

    DynamicImage::ImageRgba8(result)
}

pub fn create_collage(images: &Vec<DynamicImage>, params: &CollageParams) -> DynamicImage {
    let gap = params.gap;
    let mut result = RgbaImage::from_pixel(params.width, params.height, Rgba([255, 255, 255, 255]));

    match params.layout.as_str() {
        "grid" => {
            let count = images.len().max(1);
            let cols = (count as f32).sqrt().ceil() as usize;
            let rows = (count + cols - 1) / cols;

            let cell_w = (params.width - gap * (cols as u32 + 1)) / cols as u32;
            let cell_h = (params.height - gap * (rows as u32 + 1)) / rows as u32;

            for (i, img) in images.iter().enumerate() {
                let col = i % cols;
                let row = i / cols;
                let x = gap + col as u32 * (cell_w + gap);
                let y = gap + row as u32 * (cell_h + gap);

                let resized = img.resize(cell_w, cell_h, image::imageops::FilterType::Lanczos3);
                let (rw, rh) = resized.dimensions();
                let offset_x = x + (cell_w - rw) / 2;
                let offset_y = y + (cell_h - rh) / 2;

                for ry in 0..rh {
                    for rx in 0..rw {
                        let px = offset_x + rx;
                        let py = offset_y + ry;
                        if px < params.width && py < params.height {
                            result.put_pixel(px, py, *resized.to_rgba8().get_pixel(rx, ry));
                        }
                    }
                }
            }
        }
        "horizontal" => {
            let count = images.len().max(1) as u32;
            let cell_w = (params.width - gap * (count + 1)) / count;
            let cell_h = params.height - gap * 2;

            for (i, img) in images.iter().enumerate() {
                let x = gap + i as u32 * (cell_w + gap);
                let y = gap;

                let resized = img.resize(cell_w, cell_h, image::imageops::FilterType::Lanczos3);
                let (rw, rh) = resized.dimensions();
                let offset_y = y + (cell_h - rh) / 2;

                for ry in 0..rh {
                    for rx in 0..rw {
                        let px = x + rx;
                        let py = offset_y + ry;
                        if px < params.width && py < params.height {
                            result.put_pixel(px, py, *resized.to_rgba8().get_pixel(rx, ry));
                        }
                    }
                }
            }
        }
        "vertical" => {
            let count = images.len().max(1) as u32;
            let cell_w = params.width - gap * 2;
            let cell_h = (params.height - gap * (count + 1)) / count;

            for (i, img) in images.iter().enumerate() {
                let x = gap;
                let y = gap + i as u32 * (cell_h + gap);

                let resized = img.resize(cell_w, cell_h, image::imageops::FilterType::Lanczos3);
                let (rw, rh) = resized.dimensions();
                let offset_x = x + (cell_w - rw) / 2;

                for ry in 0..rh {
                    for rx in 0..rw {
                        let px = offset_x + rx;
                        let py = y + ry;
                        if px < params.width && py < params.height {
                            result.put_pixel(px, py, *resized.to_rgba8().get_pixel(rx, ry));
                        }
                    }
                }
            }
        }
        _ => {}
    }

    DynamicImage::ImageRgba8(result)
}
