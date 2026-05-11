use ab_glyph::{Font, FontArc, Glyph, Point, PxScale};
use image::{DynamicImage, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextParams {
    pub text: String,
    pub x: i32,
    pub y: i32,
    pub color: String,
    #[serde(default = "default_font_size")]
    pub font_size: u32,
    #[serde(default)]
    pub background_color: Option<String>,
    #[serde(default = "default_text_align")]
    pub align: String,
    #[serde(default = "default_max_width")]
    pub max_width: Option<u32>,
}

fn default_font_size() -> u32 {
    24
}

fn default_text_align() -> String {
    "left".to_string()
}

fn default_max_width() -> Option<u32> {
    None
}

// Try to load a system font once, fallback to bitmap font
fn font() -> Option<FontArc> {
    static FONT: OnceLock<Option<FontArc>> = OnceLock::new();
    FONT.get_or_init(|| {
        let candidates: Vec<&str> = vec![
            "/usr/share/fonts/google-carlito-fonts/Carlito-Regular.ttf",
            "/usr/share/fonts/google-noto/NotoSans-Regular.ttf",
            "/usr/share/fonts/liberation-sans-fonts/LiberationSans-Regular.ttf",
            "/usr/share/fonts/adwaita-sans-fonts/AdwaitaSans-Regular.ttf",
            "/usr/share/fonts/open-sans/OpenSans-Regular.ttf",
        ];
        for path in candidates {
            if let Ok(bytes) = std::fs::read(path) {
                return FontArc::try_from_vec(bytes).ok();
            }
        }
        None
    })
    .clone()
}

fn layout_glyphs(font: &impl Font, scale: PxScale, text: &str) -> Vec<Glyph> {
    let mut glyphs = Vec::new();
    let mut caret = Point { x: 0.0, y: 0.0 };
    for c in text.chars() {
        if c == '\n' {
            caret.x = 0.0;
            caret.y += scale.y * 1.2;
            continue;
        }
        let g = font.glyph_id(c).with_scale_and_position(scale, caret);
        let h_advance = font.h_advance_unscaled(g.id) * scale.x / font.units_per_em().unwrap_or(1000.0);
        caret.x += h_advance;
        glyphs.push(g);
    }
    glyphs
}

fn measure_width(font: &impl Font, scale: PxScale, text: &str) -> f32 {
    let glyphs = layout_glyphs(font, scale, text.split('\n').next().unwrap_or(""));
    let mut max_x = 0.0f32;
    let mut caret = 0.0f32;
    for g in &glyphs {
        let h_advance = font.h_advance_unscaled(g.id) * scale.x / font.units_per_em().unwrap_or(1000.0);
        max_x = max_x.max(caret + h_advance);
        caret += h_advance;
    }
    max_x.ceil()
}

pub fn draw_text(img: &DynamicImage, params: &TextParams) -> DynamicImage {
    let mut canvas = img.to_rgba8().clone();
    let color = super::hex_to_rgba(&params.color).unwrap_or(Rgba([0, 0, 0, 255]));
    let scale = PxScale::from(params.font_size.max(8) as f32);

    let lines: Vec<&str> = params.text.split('\n').collect();
    let line_height = scale.y * 1.2;

    if let Some(f) = font() {
        // ab_glyph rendering
        let line_widths: Vec<f32> = lines.iter().map(|l| measure_width(&f, scale, l)).collect();
        let max_width = line_widths.iter().copied().fold(0.0f32, f32::max);

        let _total_height = lines.len() as f32 * line_height;

        let offset_x = match params.align.as_str() {
            "center" => params.x as f32 - max_width / 2.0,
            "right" => params.x as f32 - max_width,
            _ => params.x as f32,
        };
        let offset_y = params.y as f32;

        for (i, line) in lines.iter().enumerate() {
            let line_offset = match params.align.as_str() {
                "center" => offset_x + (max_width - line_widths[i]) / 2.0,
                "right" => offset_x + (max_width - line_widths[i]),
                _ => offset_x,
            };
            let _base = Point {
                x: line_offset,
                y: offset_y + i as f32 * line_height,
            };

            let glyphs: Vec<Glyph> = layout_glyphs(&f, scale, line);

            // Draw each glyph
            for glyph in &glyphs {
                if let Some(outlined) = f.outline_glyph(glyph.clone()) {
                    let bounds = outlined.px_bounds();
                    let min_x = bounds.min.x as i32;
                    let min_y = bounds.min.y as i32;

                    let cx = (color[0] as f32) / 255.0;
                    let cy = (color[1] as f32) / 255.0;
                    let cz = (color[2] as f32) / 255.0;
                    let ca = (color[3] as f32) / 255.0;

                    outlined.draw(|dx, dy, coverage| {
                        let px = min_x + dx as i32;
                        let py = min_y + dy as i32;
                        if px >= 0 && py >= 0 && px < canvas.width() as i32 && py < canvas.height() as i32 {
                            let existing = canvas.get_pixel(px as u32, py as u32);
                            let a = coverage * ca;
                            let inv_a = 1.0 - a;
                            let r = (existing[0] as f32 * inv_a + 255.0 * cx * a) as u8;
                            let g = (existing[1] as f32 * inv_a + 255.0 * cy * a) as u8;
                            let b = (existing[2] as f32 * inv_a + 255.0 * cz * a) as u8;
                            let alpha = (existing[3] as f32 * inv_a + 255.0 * a) as u8;
                            canvas.put_pixel(px as u32, py as u32, Rgba([r, g, b, alpha]));
                        }
                    });
                }
            }
        }
    } else {
        // Fallback to bitmap font
        let char_width = ((params.font_size as f32 / 16.0) * 5.0) as i32;
        let line_h = line_height as i32;

        for (line_idx, line) in lines.iter().enumerate() {
            let line_w = line.len() as i32 * char_width;
            let start_x = match params.align.as_str() {
                "center" => params.x - line_w / 2,
                "right" => params.x - line_w,
                _ => params.x,
            };
            let start_y = params.y + line_idx as i32 * line_h;

            for (char_idx, c) in line.chars().enumerate() {
                let cx = start_x + char_idx as i32 * char_width;
                let cy = start_y;
                draw_char_bitmap(&mut canvas, c, cx, cy, color, params.font_size);
            }
        }
    }

    DynamicImage::ImageRgba8(canvas)
}

// ─── Bitmap Font Fallback ───

fn draw_char_bitmap(img: &mut RgbaImage, c: char, x: i32, y: i32, color: Rgba<u8>, size: u32) {
    let scale = size.max(8) as f32 / 16.0;
    let pattern = get_char_pattern(c);

    for (row_idx, row) in pattern.iter().enumerate() {
        for (col_idx, &pixel) in row.iter().enumerate() {
            if pixel {
                let px = x + (col_idx as f32 * scale) as i32;
                let py = y + (row_idx as f32 * scale) as i32;

                if scale > 1.0 {
                    for dy in 0..scale.ceil() as i32 {
                        for dx in 0..scale.ceil() as i32 {
                            let fx = px + dx;
                            let fy = py + dy;
                            if fx >= 0 && fy >= 0 && fx < img.width() as i32 && fy < img.height() as i32 {
                                img.put_pixel(fx as u32, fy as u32, color);
                            }
                        }
                    }
                } else {
                    if px >= 0 && py >= 0 && px < img.width() as i32 && py < img.height() as i32 {
                        img.put_pixel(px as u32, py as u32, color);
                    }
                }
            }
        }
    }
}

pub fn get_char_pattern(c: char) -> Vec<Vec<bool>> {
    let c = c.to_ascii_uppercase();
    match c {
        'A' => vec![vec![false, true, true, false], vec![true, false, false, true], vec![true, true, true, true], vec![true, false, false, true], vec![true, false, false, true]],
        'B' => vec![vec![true, true, true, false], vec![true, false, false, true], vec![true, true, true, false], vec![true, false, false, true], vec![true, true, true, false]],
        'C' => vec![vec![false, true, true, true], vec![true, false, false, false], vec![true, false, false, false], vec![true, false, false, false], vec![false, true, true, true]],
        'D' => vec![vec![true, true, true, false], vec![true, false, false, true], vec![true, false, false, true], vec![true, false, false, true], vec![true, true, true, false]],
        'E' => vec![vec![true, true, true, true], vec![true, false, false, false], vec![true, true, true, false], vec![true, false, false, false], vec![true, true, true, true]],
        'F' => vec![vec![true, true, true, true], vec![true, false, false, false], vec![true, true, true, false], vec![true, false, false, false], vec![true, false, false, false]],
        'G' => vec![vec![false, true, true, true], vec![true, false, false, false], vec![true, false, true, true], vec![true, false, false, true], vec![false, true, true, true]],
        'H' => vec![vec![true, false, false, true], vec![true, false, false, true], vec![true, true, true, true], vec![true, false, false, true], vec![true, false, false, true]],
        'I' => vec![vec![true, true, true], vec![false, true, false], vec![false, true, false], vec![false, true, false], vec![true, true, true]],
        'J' => vec![vec![false, false, true, true], vec![false, false, false, true], vec![false, false, false, true], vec![true, false, false, true], vec![false, true, true, false]],
        'K' => vec![vec![true, false, false, true], vec![true, false, true, false], vec![true, true, false, false], vec![true, false, true, false], vec![true, false, false, true]],
        'L' => vec![vec![true, false, false, false], vec![true, false, false, false], vec![true, false, false, false], vec![true, false, false, false], vec![true, true, true, true]],
        'M' => vec![vec![true, false, false, false, true], vec![true, true, false, true, true], vec![true, false, true, false, true], vec![true, false, false, false, true], vec![true, false, false, false, true]],
        'N' => vec![vec![true, false, false, false, true], vec![true, true, false, false, true], vec![true, false, true, false, true], vec![true, false, false, true, true], vec![true, false, false, false, true]],
        'O' => vec![vec![false, true, true, false], vec![true, false, false, true], vec![true, false, false, true], vec![true, false, false, true], vec![false, true, true, false]],
        'P' => vec![vec![true, true, true, false], vec![true, false, false, true], vec![true, true, true, false], vec![true, false, false, false], vec![true, false, false, false]],
        'Q' => vec![vec![false, true, true, false], vec![true, false, false, true], vec![true, false, false, true], vec![true, false, true, false], vec![false, true, false, true]],
        'R' => vec![vec![true, true, true, false], vec![true, false, false, true], vec![true, true, true, false], vec![true, false, true, false], vec![true, false, false, true]],
        'S' => vec![vec![false, true, true, true], vec![true, false, false, false], vec![false, true, true, false], vec![false, false, false, true], vec![true, true, true, false]],
        'T' => vec![vec![true, true, true, true, true], vec![false, false, true, false, false], vec![false, false, true, false, false], vec![false, false, true, false, false], vec![false, false, true, false, false]],
        'U' => vec![vec![true, false, false, true], vec![true, false, false, true], vec![true, false, false, true], vec![true, false, false, true], vec![false, true, true, false]],
        'V' => vec![vec![true, false, false, false, true], vec![true, false, false, false, true], vec![true, false, false, false, true], vec![false, true, false, true, false], vec![false, false, true, false, false]],
        'W' => vec![vec![true, false, false, false, true], vec![true, false, false, false, true], vec![true, false, true, false, true], vec![true, true, false, true, true], vec![true, false, false, false, true]],
        'X' => vec![vec![true, false, false, false, true], vec![false, true, false, true, false], vec![false, false, true, false, false], vec![false, true, false, true, false], vec![true, false, false, false, true]],
        'Y' => vec![vec![true, false, false, false, true], vec![false, true, false, true, false], vec![false, false, true, false, false], vec![false, false, true, false, false], vec![false, false, true, false, false]],
        'Z' => vec![vec![true, true, true, true, true], vec![false, false, false, true, false], vec![false, false, true, false, false], vec![false, true, false, false, false], vec![true, true, true, true, true]],
        '0' => vec![vec![false, true, true, false], vec![true, false, false, true], vec![true, false, false, true], vec![true, false, false, true], vec![false, true, true, false]],
        '1' => vec![vec![false, true, false], vec![true, true, false], vec![false, true, false], vec![false, true, false], vec![true, true, true]],
        '2' => vec![vec![false, true, true, false], vec![true, false, false, true], vec![false, false, true, false], vec![false, true, false, false], vec![true, true, true, true]],
        '3' => vec![vec![true, true, true, false], vec![false, false, false, true], vec![false, true, true, false], vec![false, false, false, true], vec![true, true, true, false]],
        '4' => vec![vec![true, false, false, true], vec![true, false, false, true], vec![true, true, true, true], vec![false, false, false, true], vec![false, false, false, true]],
        '5' => vec![vec![true, true, true, true], vec![true, false, false, false], vec![true, true, true, false], vec![false, false, false, true], vec![true, true, true, false]],
        '6' => vec![vec![false, true, true, true], vec![true, false, false, false], vec![true, true, true, false], vec![true, false, false, true], vec![false, true, true, false]],
        '7' => vec![vec![true, true, true, true], vec![false, false, false, true], vec![false, false, true, false], vec![false, true, false, false], vec![false, true, false, false]],
        '8' => vec![vec![false, true, true, false], vec![true, false, false, true], vec![false, true, true, false], vec![true, false, false, true], vec![false, true, true, false]],
        '9' => vec![vec![false, true, true, false], vec![true, false, false, true], vec![false, true, true, true], vec![false, false, false, true], vec![false, true, true, false]],
        ' ' => vec![vec![false, false, false], vec![false, false, false], vec![false, false, false], vec![false, false, false], vec![false, false, false]],
        '.' => vec![vec![false], vec![false], vec![false], vec![false], vec![true]],
        ',' => vec![vec![false], vec![false], vec![false], vec![false], vec![true], vec![true]],
        '!' => vec![vec![true], vec![true], vec![true], vec![false], vec![true]],
        '?' => vec![vec![false, true, true, false], vec![true, false, false, true], vec![false, false, true, false], vec![false, false, false, false], vec![false, false, true, false]],
        '-' => vec![vec![false, false, false], vec![false, false, false], vec![true, true, true], vec![false, false, false], vec![false, false, false]],
        '_' => vec![vec![false, false, false], vec![false, false, false], vec![false, false, false], vec![false, false, false], vec![true, true, true]],
        _ => vec![vec![true, true, true], vec![true, false, true], vec![true, false, true], vec![true, false, true], vec![true, true, true]],
    }
}
