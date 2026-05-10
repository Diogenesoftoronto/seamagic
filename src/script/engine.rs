use crate::operations::*;
use image::DynamicImage;

#[derive(Debug, Clone)]
pub enum ScriptOp {
    Crop { x: u32, y: u32, width: u32, height: u32 },
    CropCenter { width: u32, height: u32 },
    SmartCrop { aspect_ratio: Option<f32>, target_width: Option<u32>, target_height: Option<u32> },
    Resize { width: Option<u32>, height: Option<u32>, mode: String, filter: String },
    Thumbnail { size: u32 },
    Rotate { degrees: f32, background_color: Option<String> },
    Flip { horizontal: bool, vertical: bool },
    Blur { sigma: f32 },
    Brightness { value: f32 },
    Contrast { value: f32 },
    Grayscale,
    Tint { color: String, amount: f32 },
    Vignette { strength: f32, radius: f32, color: Option<String> },
    Duotone { shadow_color: String, highlight_color: String },
    DrawText { text: String, x: i32, y: i32, color: String, font_size: u32 },
    DrawRectangle { x: i32, y: i32, width: u32, height: u32, color: String, filled: bool, border_radius: u32 },
    DrawCircle { x: i32, y: i32, radius: u32, color: String, filled: bool },
    DrawLine { x1: i32, y1: i32, x2: i32, y2: i32, color: String, thickness: u32 },
    DrawPolygon { points: Vec<[i32; 2]>, color: String, filled: bool },
    Overlay { overlay_image: String, x: i32, y: i32, opacity: f32 },
    ShapeMask { shape: String, x: i32, y: i32, width: u32, height: u32, feather: bool },
    DropShadow { offset_x: i32, offset_y: i32, blur_radius: u32, color: String },
    Frame { width: u32, height: u32, frame_width: u32, frame_color: String },
    Gradient { width: u32, height: u32, start_color: String, end_color: String, angle: f32 },
    NewCanvas { width: u32, height: u32, color: String },
}

fn parse_op(line: &str) -> Result<ScriptOp, String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty operation".to_string());
    }

    let op = parts[0].to_lowercase();
    let mut kwargs: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    for part in &parts[1..] {
        if let Some(eq_pos) = part.find('=') {
            let key = part[..eq_pos].to_string();
            let value = part[eq_pos + 1..].to_string();
            kwargs.insert(key, value);
        }
    }

    fn get_str(kw: &std::collections::HashMap<String, String>, k: &str) -> Result<String, String> {
        kw.get(k).cloned().ok_or_else(|| format!("Missing parameter: {}", k))
    }
    fn get_str_opt(kw: &std::collections::HashMap<String, String>, k: &str, d: &str) -> String {
        kw.get(k).cloned().unwrap_or_else(|| d.to_string())
    }
    fn get_u32(kw: &std::collections::HashMap<String, String>, k: &str) -> Result<u32, String> {
        kw.get(k).ok_or_else(|| format!("Missing parameter: {}", k))?
            .parse::<u32>().map_err(|e| format!("Invalid {}: {}", k, e))
    }
    fn get_u32_opt(kw: &std::collections::HashMap<String, String>, k: &str, d: u32) -> u32 {
        kw.get(k).and_then(|v| v.parse().ok()).unwrap_or(d)
    }
    fn get_i32(kw: &std::collections::HashMap<String, String>, k: &str) -> Result<i32, String> {
        kw.get(k).ok_or_else(|| format!("Missing parameter: {}", k))?
            .parse::<i32>().map_err(|e| format!("Invalid {}: {}", k, e))
    }
    fn get_i32_opt(kw: &std::collections::HashMap<String, String>, k: &str, d: i32) -> i32 {
        kw.get(k).and_then(|v| v.parse().ok()).unwrap_or(d)
    }
    fn get_f32(kw: &std::collections::HashMap<String, String>, k: &str) -> Result<f32, String> {
        kw.get(k).ok_or_else(|| format!("Missing parameter: {}", k))?
            .parse::<f32>().map_err(|e| format!("Invalid {}: {}", k, e))
    }
    fn get_f32_opt(kw: &std::collections::HashMap<String, String>, k: &str, d: f32) -> f32 {
        kw.get(k).and_then(|v| v.parse().ok()).unwrap_or(d)
    }
    fn get_bool_opt(kw: &std::collections::HashMap<String, String>, k: &str, d: bool) -> bool {
        kw.get(k).map(|v| v.as_str() == "true" || v.as_str() == "1").unwrap_or(d)
    }
    fn get_opt_opt(kw: &std::collections::HashMap<String, String>, k: &str) -> Option<String> {
        kw.get(k).cloned()
    }
    fn get_opt_f32(kw: &std::collections::HashMap<String, String>, k: &str) -> Option<f32> {
        kw.get(k).and_then(|v| v.parse().ok())
    }
    fn get_opt_u32(kw: &std::collections::HashMap<String, String>, k: &str) -> Option<u32> {
        kw.get(k).and_then(|v| v.parse().ok())
    }

    match op.as_str() {
        "crop" => Ok(ScriptOp::Crop {
            x: get_u32(&kwargs, "x")?,
            y: get_u32(&kwargs, "y")?,
            width: get_u32(&kwargs, "width")?,
            height: get_u32(&kwargs, "height")?,
        }),
        "crop_center" => Ok(ScriptOp::CropCenter {
            width: get_u32(&kwargs, "width")?,
            height: get_u32(&kwargs, "height")?,
        }),
        "smart_crop" => Ok(ScriptOp::SmartCrop {
            aspect_ratio: get_opt_f32(&kwargs, "aspect_ratio"),
            target_width: get_opt_u32(&kwargs, "target_width"),
            target_height: get_opt_u32(&kwargs, "target_height"),
        }),
        "resize" => Ok(ScriptOp::Resize {
            width: get_opt_u32(&kwargs, "width"),
            height: get_opt_u32(&kwargs, "height"),
            mode: get_str_opt(&kwargs, "mode", "exact"),
            filter: get_str_opt(&kwargs, "filter", "lanczos3"),
        }),
        "thumbnail" => Ok(ScriptOp::Thumbnail { size: get_u32(&kwargs, "size")? }),
        "rotate" => Ok(ScriptOp::Rotate {
            degrees: get_f32(&kwargs, "degrees")?,
            background_color: get_opt_opt(&kwargs, "bg"),
        }),
        "flip" => Ok(ScriptOp::Flip {
            horizontal: get_bool_opt(&kwargs, "h", false),
            vertical: get_bool_opt(&kwargs, "v", false),
        }),
        "blur" => Ok(ScriptOp::Blur { sigma: get_f32(&kwargs, "sigma")? }),
        "brightness" => Ok(ScriptOp::Brightness { value: get_f32(&kwargs, "value")? }),
        "contrast" => Ok(ScriptOp::Contrast { value: get_f32(&kwargs, "value")? }),
        "grayscale" | "greyscale" => Ok(ScriptOp::Grayscale),
        "tint" => Ok(ScriptOp::Tint {
            color: get_str(&kwargs, "color")?,
            amount: get_f32_opt(&kwargs, "amount", 0.5),
        }),
        "vignette" => Ok(ScriptOp::Vignette {
            strength: get_f32_opt(&kwargs, "strength", 0.5),
            radius: get_f32_opt(&kwargs, "radius", 1.0),
            color: get_opt_opt(&kwargs, "color"),
        }),
        "duotone" => Ok(ScriptOp::Duotone {
            shadow_color: get_str(&kwargs, "shadow")?,
            highlight_color: get_str(&kwargs, "highlight")?,
        }),
        "text" => Ok(ScriptOp::DrawText {
            text: get_str(&kwargs, "text")?,
            x: get_i32(&kwargs, "x")?,
            y: get_i32(&kwargs, "y")?,
            color: get_str_opt(&kwargs, "color", "#FFFFFF"),
            font_size: get_u32_opt(&kwargs, "size", 24),
        }),
        "rect" | "rectangle" => Ok(ScriptOp::DrawRectangle {
            x: get_i32(&kwargs, "x")?,
            y: get_i32(&kwargs, "y")?,
            width: get_u32(&kwargs, "width")?,
            height: get_u32(&kwargs, "height")?,
            color: get_str(&kwargs, "color")?,
            filled: get_bool_opt(&kwargs, "filled", true),
            border_radius: get_u32_opt(&kwargs, "radius", 0),
        }),
        "circle" => Ok(ScriptOp::DrawCircle {
            x: get_i32(&kwargs, "x")?,
            y: get_i32(&kwargs, "y")?,
            radius: get_u32(&kwargs, "radius")?,
            color: get_str(&kwargs, "color")?,
            filled: get_bool_opt(&kwargs, "filled", true),
        }),
        "line" => Ok(ScriptOp::DrawLine {
            x1: get_i32(&kwargs, "x1")?,
            y1: get_i32(&kwargs, "y1")?,
            x2: get_i32(&kwargs, "x2")?,
            y2: get_i32(&kwargs, "y2")?,
            color: get_str(&kwargs, "color")?,
            thickness: get_u32_opt(&kwargs, "thickness", 1),
        }),
        "polygon" => {
            let points_str = get_str(&kwargs, "points")?;
            let points: Vec<[i32; 2]> = points_str
                .split(';')
                .filter(|s| !s.is_empty())
                .map(|p| {
                    let coords: Vec<i32> = p.split(',').filter_map(|c| c.trim().parse().ok()).collect();
                    if coords.len() >= 2 { [coords[0], coords[1]] } else { [0, 0] }
                })
                .collect();
            Ok(ScriptOp::DrawPolygon {
                points,
                color: get_str(&kwargs, "color")?,
                filled: get_bool_opt(&kwargs, "filled", true),
            })
        }
        "overlay" => Ok(ScriptOp::Overlay {
            overlay_image: get_str(&kwargs, "image")?,
            x: get_i32_opt(&kwargs, "x", 0),
            y: get_i32_opt(&kwargs, "y", 0),
            opacity: get_f32_opt(&kwargs, "opacity", 1.0),
        }),
        "shape_mask" => Ok(ScriptOp::ShapeMask {
            shape: get_str_opt(&kwargs, "shape", "circle"),
            x: get_i32_opt(&kwargs, "x", 0),
            y: get_i32_opt(&kwargs, "y", 0),
            width: get_u32(&kwargs, "width")?,
            height: get_u32(&kwargs, "height")?,
            feather: get_bool_opt(&kwargs, "feather", false),
        }),
        "drop_shadow" => Ok(ScriptOp::DropShadow {
            offset_x: get_i32_opt(&kwargs, "offset_x", 5),
            offset_y: get_i32_opt(&kwargs, "offset_y", 5),
            blur_radius: get_u32_opt(&kwargs, "blur", 10),
            color: get_str_opt(&kwargs, "color", "#00000080"),
        }),
        "frame" => Ok(ScriptOp::Frame {
            width: get_u32(&kwargs, "width")?,
            height: get_u32(&kwargs, "height")?,
            frame_width: get_u32(&kwargs, "frame_width")?,
            frame_color: get_str(&kwargs, "color")?,
        }),
        "gradient" => Ok(ScriptOp::Gradient {
            width: get_u32(&kwargs, "width")?,
            height: get_u32(&kwargs, "height")?,
            start_color: get_str(&kwargs, "start")?,
            end_color: get_str(&kwargs, "end")?,
            angle: get_f32_opt(&kwargs, "angle", 0.0),
        }),
        "canvas" => Ok(ScriptOp::NewCanvas {
            width: get_u32(&kwargs, "width")?,
            height: get_u32(&kwargs, "height")?,
            color: get_str_opt(&kwargs, "color", "#FFFFFF"),
        }),
        _ => Err(format!("Unknown operation: {}", op)),
    }
}

pub fn run_script(script: &str, input_image: Option<DynamicImage>) -> Result<DynamicImage, String> {
    let ops: Result<Vec<ScriptOp>, String> = script
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
        .map(|line| {
            let line = if let Some(pos) = line.find(" #") {
                &line[..pos]
            } else {
                line
            }.trim();
            parse_op(line)
        })
        .collect();

    let ops = ops?;
    let mut current = input_image;

    for op in ops {
        current = Some(execute_op(op, current)?);
    }

    current.ok_or_else(|| "No image produced".to_string())
}

fn execute_op(op: ScriptOp, current: Option<DynamicImage>) -> Result<DynamicImage, String> {
    let img = match op {
        ScriptOp::NewCanvas { width, height, ref color } => {
            let rgba = hex_to_rgba(color).map_err(|e| e.to_string())?;
            return Ok(DynamicImage::ImageRgba8(
                image::RgbaImage::from_pixel(width, height, rgba)
            ));
        }
        ScriptOp::Gradient { width, height, ref start_color, ref end_color, angle } => {
            return Ok(create_gradient(&GradientParams {
                width, height,
                start_color: start_color.clone(),
                end_color: end_color.clone(),
                angle,
            }));
        }
        ScriptOp::Frame { width, height, frame_width, ref frame_color } => {
            return Ok(create_frame(&FrameParams {
                width, height, frame_width,
                frame_color: frame_color.clone(),
                inner_color: None,
            }));
        }
        _ => current.ok_or("No input image available for this operation")?,
    };

    let result = match op {
        ScriptOp::Crop { x, y, width, height } => {
            crop(&img, &CropParams { x, y, width, height })
        }
        ScriptOp::CropCenter { width, height } => {
            crop_from_center(&img, &CropFromCenterParams { width, height })
        }
        ScriptOp::SmartCrop { aspect_ratio, target_width, target_height } => {
            smart_crop(&img, &SmartCropParams { aspect_ratio, target_width, target_height, focus_x: Some(0.5), focus_y: Some(0.5) })
        }
        ScriptOp::Resize { width, height, ref mode, ref filter } => {
            let mode = match mode.as_str() {
                "fit" => ResizeMode::Fit,
                "fill" => ResizeMode::Fill,
                "scale" => ResizeMode::Scale,
                _ => ResizeMode::Exact,
            };
            resize(&img, &ResizeParams { width, height, mode, filter: filter.clone() })
        }
        ScriptOp::Thumbnail { size } => {
            thumbnail(&img, &ThumbnailParams { size })
        }
        ScriptOp::Rotate { degrees, ref background_color } => {
            rotate(&img, &RotateParams { degrees, background_color: background_color.clone() })
        }
        ScriptOp::Flip { horizontal, vertical } => {
            flip(&img, &FlipParams { horizontal, vertical })
        }
        ScriptOp::Blur { sigma } => {
            blur(&img, &BlurParams { sigma })
        }
        ScriptOp::Brightness { value } => {
            brightness(&img, &BrightnessParams { value })
        }
        ScriptOp::Contrast { value } => {
            filters::contrast(&img, &ContrastParams { value })
        }
        ScriptOp::Grayscale => {
            grayscale(&img, &GrayscaleParams { mode: "luma".to_string() })
        }
        ScriptOp::Tint { ref color, amount } => {
            tint(&img, &TintParams { color: color.clone(), amount })
        }
        ScriptOp::Vignette { strength, radius, ref color } => {
            vignette(&img, &VignetteParams { strength, radius, color: color.clone() })
        }
        ScriptOp::Duotone { ref shadow_color, ref highlight_color } => {
            duotone(&img, &DuotoneParams { shadow_color: shadow_color.clone(), highlight_color: highlight_color.clone() })
        }
        ScriptOp::DrawText { ref text, x, y, ref color, font_size } => {
            draw_text(&img, &TextParams {
                text: text.clone(), x, y,
                color: color.clone(), font_size,
                align: "left".to_string(),
                background_color: None,
                max_width: None,
            })
        }
        ScriptOp::DrawRectangle { x, y, width, height, ref color, filled, border_radius } => {
            draw_rectangle(&img, &RectangleParams {
                x, y, width, height,
                color: color.clone(), filled,
                border_radius,
                stroke_width: 0,
                stroke_color: None,
            })
        }
        ScriptOp::DrawCircle { x, y, radius, ref color, filled } => {
            draw_circle(&img, &CircleMaskParams {
                x, y, radius,
                color: color.clone(),
                filled,
                stroke_width: 0,
                stroke_color: None,
            })
        }
        ScriptOp::DrawLine { x1, y1, x2, y2, ref color, thickness } => {
            draw_line(&img, &LineParams { x1, y1, x2, y2, color: color.clone(), thickness })
        }
        ScriptOp::DrawPolygon { ref points, ref color, filled } => {
            draw_polygon(&img, &PolygonParams {
                points: points.clone(),
                color: color.clone(),
                filled,
                stroke_width: 0,
                stroke_color: None,
            })
        }
        ScriptOp::Overlay { ref overlay_image, x, y, opacity } => {
            let overlay_img = load_from_base64(overlay_image).map_err(|e| e.to_string())?;
            overlay(&img, &overlay_img, &OverlayParams {
                base_image: String::new(),
                overlay_image: overlay_image.clone(),
                x, y,
                opacity: Some(opacity),
                blend_mode: None,
            })
        }
        ScriptOp::ShapeMask { ref shape, x, y, width, height, feather } => {
            let mask = create_shape_mask(&img, &ShapeMaskParams {
                shape: shape.clone(), x, y, width, height,
                feather,
                feather_radius: 10,
            });
            apply_mask(&img, &mask)
        }
        ScriptOp::DropShadow { offset_x, offset_y, blur_radius, ref color } => {
            drop_shadow(&img, &ShadowParams { offset_x, offset_y, blur_radius, color: color.clone() })
        }
        _ => return Err("Unexpected operation in execute_op".to_string()),
    };

    Ok(result)
}
