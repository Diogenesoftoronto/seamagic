use image::{DynamicImage, Rgba, RgbaImage, GenericImage};
use serde::{Deserialize, Serialize};

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

fn draw_char(img: &mut RgbaImage, c: char, x: i32, y: i32, color: Rgba<u8>, size: u32) {
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
        'A' => vec![
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![true, true, true, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
        ],
        'B' => vec![
            vec![true, true, true, false],
            vec![true, false, false, true],
            vec![true, true, true, false],
            vec![true, false, false, true],
            vec![true, true, true, false],
        ],
        'C' => vec![
            vec![false, true, true, true],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![false, true, true, true],
        ],
        'D' => vec![
            vec![true, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, true, true, false],
        ],
        'E' => vec![
            vec![true, true, true, true],
            vec![true, false, false, false],
            vec![true, true, true, false],
            vec![true, false, false, false],
            vec![true, true, true, true],
        ],
        'F' => vec![
            vec![true, true, true, true],
            vec![true, false, false, false],
            vec![true, true, true, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
        ],
        'G' => vec![
            vec![false, true, true, true],
            vec![true, false, false, false],
            vec![true, false, true, true],
            vec![true, false, false, true],
            vec![false, true, true, true],
        ],
        'H' => vec![
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, true, true, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
        ],
        'I' => vec![
            vec![true, true, true],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![true, true, true],
        ],
        'J' => vec![
            vec![false, false, true, true],
            vec![false, false, false, true],
            vec![false, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
        ],
        'K' => vec![
            vec![true, false, false, true],
            vec![true, false, true, false],
            vec![true, true, false, false],
            vec![true, false, true, false],
            vec![true, false, false, true],
        ],
        'L' => vec![
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, true, true, true],
        ],
        'M' => vec![
            vec![true, false, false, false, true],
            vec![true, true, false, true, true],
            vec![true, false, true, false, true],
            vec![true, false, false, false, true],
            vec![true, false, false, false, true],
        ],
        'N' => vec![
            vec![true, false, false, false, true],
            vec![true, true, false, false, true],
            vec![true, false, true, false, true],
            vec![true, false, false, true, true],
            vec![true, false, false, false, true],
        ],
        'O' => vec![
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
        ],
        'P' => vec![
            vec![true, true, true, false],
            vec![true, false, false, true],
            vec![true, true, true, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
        ],
        'Q' => vec![
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, true, false],
            vec![false, true, false, true],
        ],
        'R' => vec![
            vec![true, true, true, false],
            vec![true, false, false, true],
            vec![true, true, true, false],
            vec![true, false, true, false],
            vec![true, false, false, true],
        ],
        'S' => vec![
            vec![false, true, true, true],
            vec![true, false, false, false],
            vec![false, true, true, false],
            vec![false, false, false, true],
            vec![true, true, true, false],
        ],
        'T' => vec![
            vec![true, true, true, true, true],
            vec![false, false, true, false, false],
            vec![false, false, true, false, false],
            vec![false, false, true, false, false],
            vec![false, false, true, false, false],
        ],
        'U' => vec![
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
        ],
        'V' => vec![
            vec![true, false, false, false, true],
            vec![true, false, false, false, true],
            vec![true, false, false, false, true],
            vec![false, true, false, true, false],
            vec![false, false, true, false, false],
        ],
        'W' => vec![
            vec![true, false, false, false, true],
            vec![true, false, false, false, true],
            vec![true, false, true, false, true],
            vec![true, true, false, true, true],
            vec![true, false, false, false, true],
        ],
        'X' => vec![
            vec![true, false, false, false, true],
            vec![false, true, false, true, false],
            vec![false, false, true, false, false],
            vec![false, true, false, true, false],
            vec![true, false, false, false, true],
        ],
        'Y' => vec![
            vec![true, false, false, false, true],
            vec![false, true, false, true, false],
            vec![false, false, true, false, false],
            vec![false, false, true, false, false],
            vec![false, false, true, false, false],
        ],
        'Z' => vec![
            vec![true, true, true, true, true],
            vec![false, false, false, true, false],
            vec![false, false, true, false, false],
            vec![false, true, false, false, false],
            vec![true, true, true, true, true],
        ],
        '0' => vec![
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
        ],
        '1' => vec![
            vec![false, true, false],
            vec![true, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![true, true, true],
        ],
        '2' => vec![
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![false, false, true, false],
            vec![false, true, false, false],
            vec![true, true, true, true],
        ],
        '3' => vec![
            vec![true, true, true, false],
            vec![false, false, false, true],
            vec![false, true, true, false],
            vec![false, false, false, true],
            vec![true, true, true, false],
        ],
        '4' => vec![
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, true, true, true],
            vec![false, false, false, true],
            vec![false, false, false, true],
        ],
        '5' => vec![
            vec![true, true, true, true],
            vec![true, false, false, false],
            vec![true, true, true, false],
            vec![false, false, false, true],
            vec![true, true, true, false],
        ],
        '6' => vec![
            vec![false, true, true, true],
            vec![true, false, false, false],
            vec![true, true, true, false],
            vec![true, false, false, true],
            vec![false, true, true, false],
        ],
        '7' => vec![
            vec![true, true, true, true],
            vec![false, false, false, true],
            vec![false, false, true, false],
            vec![false, true, false, false],
            vec![false, true, false, false],
        ],
        '8' => vec![
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![false, true, true, false],
        ],
        '9' => vec![
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![false, true, true, true],
            vec![false, false, false, true],
            vec![false, true, true, false],
        ],
        ' ' => vec![
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
        ],
        '.' => vec![
            vec![false],
            vec![false],
            vec![false],
            vec![false],
            vec![true],
        ],
        ',' => vec![
            vec![false],
            vec![false],
            vec![false],
            vec![false],
            vec![true],
            vec![true],
        ],
        '!' => vec![
            vec![true],
            vec![true],
            vec![true],
            vec![false],
            vec![true],
        ],
        '?' => vec![
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![false, false, true, false],
            vec![false, false, false, false],
            vec![false, false, true, false],
        ],
        '-' => vec![
            vec![false, false, false],
            vec![false, false, false],
            vec![true, true, true],
            vec![false, false, false],
            vec![false, false, false],
        ],
        '_' => vec![
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
            vec![true, true, true],
        ],
        _ => vec![
            vec![true, true, true],
            vec![true, false, true],
            vec![true, false, true],
            vec![true, false, true],
            vec![true, true, true],
        ],
    }
}

pub fn draw_text(img: &DynamicImage, params: &TextParams) -> DynamicImage {
    let mut canvas = img.to_rgba8();
    let color = super::hex_to_rgba(&params.color).unwrap_or(Rgba([0, 0, 0, 255]));

    let char_width = ((params.font_size as f32 / 16.0) * 5.0) as i32;
    let line_height = (params.font_size as f32 * 1.3) as i32;

    let mut cx = params.x;
    let mut cy = params.y;

    for c in params.text.chars() {
        if c == '\n' {
            cx = params.x;
            cy += line_height;
            continue;
        }

        draw_char(&mut canvas, c, cx, cy, color, params.font_size);
        cx += char_width;
    }

    DynamicImage::ImageRgba8(canvas)
}
