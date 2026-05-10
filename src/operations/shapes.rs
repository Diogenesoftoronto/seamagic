use image::{DynamicImage, Rgba, ImageBuffer, GenericImage, GenericImageView};
use imageproc::drawing;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CircleMaskParams {
    pub x: i32,
    pub y: i32,
    pub radius: u32,
    pub color: String,
    #[serde(default)]
    pub stroke_width: u32,
    #[serde(default)]
    pub stroke_color: Option<String>,
    #[serde(default)]
    pub filled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RectangleParams {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub color: String,
    #[serde(default)]
    pub filled: bool,
    #[serde(default)]
    pub stroke_width: u32,
    #[serde(default)]
    pub stroke_color: Option<String>,
    #[serde(default)]
    pub border_radius: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineParams {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
    pub color: String,
    pub thickness: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolygonParams {
    pub points: Vec<[i32; 2]>,
    pub color: String,
    #[serde(default)]
    pub filled: bool,
    #[serde(default)]
    pub stroke_width: u32,
    #[serde(default)]
    pub stroke_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GradientParams {
    pub start_color: String,
    pub end_color: String,
    #[serde(default)]
    pub angle: f32,
    pub width: u32,
    pub height: u32,
}

fn draw_rounded_rect(img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, x: i32, y: i32, w: u32, h: u32, radius: u32, color: Rgba<u8>, filled: bool, stroke_width: u32) {
    let r = radius.min(w / 2).min(h / 2) as i32;

    if filled {
        for dy in 0..h as i32 {
            for dx in 0..w as i32 {
                let px = x + dx;
                let py = y + dy;
                if px < 0 || py < 0 { continue; }

                let lx = dx as i32;
                let ly = dy as i32;

                let in_rect = lx >= r && lx < (w as i32 - r) || ly >= r && ly < (h as i32 - r);
                let in_tl = (lx - r).pow(2) + (ly - r).pow(2) <= r * r;
                let in_tr = (lx - (w as i32 - r)).pow(2) + (ly - r).pow(2) <= r * r;
                let in_bl = (lx - r).pow(2) + (ly - (h as i32 - r)).pow(2) <= r * r;
                let in_br = (lx - (w as i32 - r)).pow(2) + (ly - (h as i32 - r)).pow(2) <= r * r;

                if in_rect || in_tl || in_tr || in_bl || in_br {
                    if px < img.width() as i32 && py < img.height() as i32 {
                        img.put_pixel(px as u32, py as u32, color);
                    }
                }
            }
        }
    }

    if stroke_width > 0 {
        let stroke = Rgba([255, 0, 0, 255]);
        for dy in 0..h as i32 {
            for dx in 0..w as i32 {
                let px = x + dx;
                let py = y + dy;
                if px < 0 || py < 0 { continue; }

                let lx = dx as i32;
                let ly = dy as i32;

                let on_left = lx < stroke_width as i32;
                let on_right = lx >= (w as i32 - stroke_width as i32);
                let on_top = ly < stroke_width as i32;
                let on_bottom = ly >= (h as i32 - stroke_width as i32);

                let tl_corner = lx < r && ly < r && (lx - r).pow(2) + (ly - r).pow(2) > (r - stroke_width as i32).pow(2);
                let tr_corner = lx >= (w as i32 - r) && ly < r && (lx - (w as i32 - r)).pow(2) + (ly - r).pow(2) > (r - stroke_width as i32).pow(2);
                let bl_corner = lx < r && ly >= (h as i32 - r) && (lx - r).pow(2) + (ly - (h as i32 - r)).pow(2) > (r - stroke_width as i32).pow(2);
                let br_corner = lx >= (w as i32 - r) && ly >= (h as i32 - r) && (lx - (w as i32 - r)).pow(2) + (ly - (h as i32 - r)).pow(2) > (r - stroke_width as i32).pow(2);

                let on_edge = (on_left || on_right) && (ly >= r && ly < (h as i32 - r));
                let on_top_bottom = (on_top || on_bottom) && (lx >= r && lx < (w as i32 - r));

                if (on_edge || on_top_bottom || tl_corner || tr_corner || bl_corner || br_corner)
                    && px < img.width() as i32 && py < img.height() as i32 {
                    img.put_pixel(px as u32, py as u32, stroke);
                }
            }
        }
    }
}

pub fn draw_circle(img: &DynamicImage, params: &CircleMaskParams) -> DynamicImage {
    let mut canvas = img.to_rgba8();
    let color = super::hex_to_rgba(&params.color).unwrap_or(Rgba([0, 0, 0, 255]));

    for dy in -(params.radius as i32)..=(params.radius as i32) {
        for dx in -(params.radius as i32)..=(params.radius as i32) {
            let dist = ((dx * dx + dy * dy) as f32).sqrt();
            if dist <= params.radius as f32 {
                if params.filled || dist >= (params.radius - params.stroke_width) as f32 {
                    let px = params.x + dx;
                    let py = params.y + dy;
                    if px >= 0 && py >= 0 && px < canvas.width() as i32 && py < canvas.height() as i32 {
                        canvas.put_pixel(px as u32, py as u32, color);
                    }
                }
            }
        }
    }

    DynamicImage::ImageRgba8(canvas)
}

pub fn draw_rectangle(img: &DynamicImage, params: &RectangleParams) -> DynamicImage {
    let mut canvas = img.to_rgba8();
    let color = super::hex_to_rgba(&params.color).unwrap_or(Rgba([0, 0, 0, 255]));

    if params.border_radius > 0 {
        draw_rounded_rect(&mut canvas, params.x, params.y, params.width, params.height, params.border_radius, color, params.filled, params.stroke_width);
    } else {
        for dy in 0..params.height as i32 {
            for dx in 0..params.width as i32 {
                let px = params.x + dx;
                let py = params.y + dy;
                if px < 0 || py < 0 || px >= canvas.width() as i32 || py >= canvas.height() as i32 {
                    continue;
                }

                let on_edge = dx < params.stroke_width as i32
                    || dx >= (params.width as i32 - params.stroke_width as i32)
                    || dy < params.stroke_width as i32
                    || dy >= (params.height as i32 - params.stroke_width as i32);

                if params.filled || on_edge {
                    canvas.put_pixel(px as u32, py as u32, color);
                }
            }
        }
    }

    DynamicImage::ImageRgba8(canvas)
}

pub fn draw_line(img: &DynamicImage, params: &LineParams) -> DynamicImage {
    let mut canvas = img.to_rgba8();
    let color = super::hex_to_rgba(&params.color).unwrap_or(Rgba([0, 0, 0, 255]));

    let dx = (params.x2 - params.x1).abs();
    let dy = (params.y2 - params.y1).abs();
    let sx = if params.x1 < params.x2 { 1 } else { -1 };
    let sy = if params.y1 < params.y2 { 1 } else { -1 };
    let mut err = dx - dy;
    let mut x = params.x1;
    let mut y = params.y1;

    let thickness = params.thickness.max(1) as i32;
    let half_thick = thickness / 2;

    loop {
        for ty in -half_thick..=half_thick {
            for tx in -half_thick..=half_thick {
                let px = x + tx;
                let py = y + ty;
                if px >= 0 && py >= 0 && px < canvas.width() as i32 && py < canvas.height() as i32 {
                    canvas.put_pixel(px as u32, py as u32, color);
                }
            }
        }

        if x == params.x2 && y == params.y2 { break; }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }

    DynamicImage::ImageRgba8(canvas)
}

pub fn draw_polygon(img: &DynamicImage, params: &PolygonParams) -> DynamicImage {
    let mut canvas = img.to_rgba8();
    let color = super::hex_to_rgba(&params.color).unwrap_or(Rgba([0, 0, 0, 255]));

    if params.points.len() < 3 {
        return DynamicImage::ImageRgba8(canvas);
    }

    if params.filled {
        let poly_points: Vec<imageproc::point::Point<i32>> = params.points
            .iter()
            .map(|p| imageproc::point::Point::new(p[0], p[1]))
            .collect();
        drawing::draw_polygon_mut(&mut canvas, &poly_points, color);
    }

    if params.stroke_width > 0 || !params.filled {
        for i in 0..params.points.len() {
            let p1 = params.points[i];
            let p2 = params.points[(i + 1) % params.points.len()];
            drawing::draw_line_segment_mut(&mut canvas, (p1[0] as f32, p1[1] as f32), (p2[0] as f32, p2[1] as f32), color);
        }
    }

    DynamicImage::ImageRgba8(canvas)
}

pub fn create_gradient(params: &GradientParams) -> DynamicImage {
    let width = params.width.max(1);
    let height = params.height.max(1);
    let mut img = ImageBuffer::new(width, height);

    let start = super::hex_to_rgba(&params.start_color).unwrap_or(Rgba([0, 0, 0, 255]));
    let end = super::hex_to_rgba(&params.end_color).unwrap_or(Rgba([255, 255, 255, 255]));

    let angle_rad = params.angle.to_radians();
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();

    for y in 0..height {
        for x in 0..width {
            let nx = (x as f32 - width as f32 / 2.0) / (width as f32 / 2.0);
            let ny = (y as f32 - height as f32 / 2.0) / (height as f32 / 2.0);
            let t = (nx * cos_a + ny * sin_a + 1.0) / 2.0;
            let t = t.clamp(0.0, 1.0);

            let r = (start[0] as f32 * (1.0 - t) + end[0] as f32 * t) as u8;
            let g = (start[1] as f32 * (1.0 - t) + end[1] as f32 * t) as u8;
            let b = (start[2] as f32 * (1.0 - t) + end[2] as f32 * t) as u8;
            let a = (start[3] as f32 * (1.0 - t) + end[3] as f32 * t) as u8;

            img.put_pixel(x, y, Rgba([r, g, b, a]));
        }
    }

    DynamicImage::ImageRgba8(img)
}
