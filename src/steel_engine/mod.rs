use image::{DynamicImage, GenericImageView, ImageFormat};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use steel::steel_vm::engine::Engine;
use steel::steel_vm::register_fn::RegisterFn;
use steel::SteelVal;

use crate::operations::*;
use crate::operations::segmentation::{SegmentImageParams, RemoveBackgroundParams, segment_images_to_b64, remove_background_to_b64};

lazy_static::lazy_static! {
    static ref IMAGE_REGISTRY: Arc<Mutex<HashMap<String, DynamicImage>>> = Arc::new(Mutex::new(HashMap::new()));
}

fn with_registry<F, T>(f: F) -> T
where
    F: FnOnce(&mut HashMap<String, DynamicImage>) -> T,
{
    let mut reg = IMAGE_REGISTRY.lock().unwrap();
    f(&mut *reg)
}

fn b64_to_img(b64: &str) -> Result<DynamicImage, String> {
    load_from_base64(b64).map_err(|e| format!(
        "Failed to decode base64 image ({} bytes received). Is this a valid base64 PNG/JPG? Error: {}",
        b64.len(), e
    ))
}

fn img_to_b64(img: &DynamicImage) -> Result<String, String> {
    save_to_base64(img, ImageFormat::Png).map_err(|e| format!(
        "Failed to encode {}x{} image to base64 PNG: {}",
        img.width(), img.height(), e
    ))
}

fn register_img(id: &str, img: DynamicImage) {
    with_registry(|reg| {
        reg.insert(id.to_string(), img);
    });
}

fn get_img(id: &str) -> Result<DynamicImage, String> {
    with_registry(|reg| {
        reg.get(id).cloned().ok_or_else(|| {
            let keys: Vec<_> = reg.keys().take(10).map(|k| k.as_str()).collect();
            if keys.is_empty() {
                format!("Image '{}' not found in registry. Registry is empty — did the function that creates images complete successfully?", id)
            } else {
                format!("Image '{}' not found in registry. Available IDs: {:?}", id, keys)
            }
        })
    })
}

fn new_id() -> String {
    format!("img-{}", uuid::Uuid::new_v4())
}

// ─── Image Operations exposed to Steel ───

fn to_f64(v: &SteelVal) -> Result<f64, String> {
    match v {
        SteelVal::IntV(i) => Ok(*i as f64),
        SteelVal::NumV(n) => Ok(*n),
        SteelVal::Rational(r) => Ok(*r.numer() as f64 / *r.denom() as f64),
        SteelVal::BigRational(r) => Ok((r.numer().to_string().parse::<f64>().unwrap_or(0.0)) / (r.denom().to_string().parse::<f64>().unwrap_or(1.0))),
        _ => Err(format!("Expected number, found: {:?}", v)),
    }
}

fn to_u32(v: &SteelVal) -> Result<u32, String> {
    Ok(to_f64(v)? as u32)
}

fn steel_crop(img_id: String, x: SteelVal, y: SteelVal, width: SteelVal, height: SteelVal) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let result = crop(&img, &CropParams {
        x: to_u32(&x)?, y: to_u32(&y)?,
        width: to_u32(&width)?, height: to_u32(&height)?,
    });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_crop_center(img_id: String, width: SteelVal, height: SteelVal) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let result = crop_from_center(&img, &CropFromCenterParams {
        width: to_u32(&width)?, height: to_u32(&height)?,
    });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_smart_crop(img_id: String, aspect_ratio: Option<f64>, target_width: Option<f64>, target_height: Option<f64>) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let result = smart_crop(&img, &SmartCropParams {
        aspect_ratio: aspect_ratio.map(|f| f as f32),
        target_width: target_width.map(|f| f as u32),
        target_height: target_height.map(|f| f as u32),
        focus_x: Some(0.5), focus_y: Some(0.5),
    });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_resize(img_id: String, width: Option<SteelVal>, height: Option<SteelVal>, mode: String) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let mode = match mode.as_str() {
        "fit" => ResizeMode::Fit,
        "fill" => ResizeMode::Fill,
        "scale" => ResizeMode::Scale,
        _ => ResizeMode::Exact,
    };
    let result = resize(&img, &ResizeParams {
        width: width.map(|v| to_u32(&v)).transpose()?,
        height: height.map(|v| to_u32(&v)).transpose()?,
        mode,
        filter: "lanczos3".to_string(),
    });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_thumbnail(img_id: String, size: f64) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let result = thumbnail(&img, &ThumbnailParams { size: size as u32 });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_rotate(img_id: String, degrees: f64, bg: Option<String>) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let result = rotate(&img, &RotateParams {
        degrees: degrees as f32,
        background_color: bg,
    });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_flip(img_id: String, h: bool, v: bool) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let result = flip(&img, &FlipParams { horizontal: h, vertical: v });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_blur(img_id: String, sigma: f64) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let result = blur(&img, &BlurParams { sigma: sigma as f32 });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_brightness(img_id: String, value: isize) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let result = brightness(&img, &BrightnessParams { value: value as f32 });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_contrast(img_id: String, value: isize) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let result = filters::contrast(&img, &ContrastParams { value: value as f32 });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_grayscale(img_id: String) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let result = grayscale(&img, &GrayscaleParams { mode: "luma".to_string() });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_tint(img_id: String, color: String, amount: f64) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let result = tint(&img, &TintParams { color, amount: amount as f32 });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_vignette(img_id: String, strength: f64, radius: f64, color: Option<String>) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let result = vignette(&img, &VignetteParams {
        strength: strength as f32,
        radius: radius as f32,
        color,
    });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_duotone(img_id: String, shadow_color: String, highlight_color: String) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let result = duotone(&img, &DuotoneParams { shadow_color, highlight_color });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_draw_text(img_id: String, text: String, x: SteelVal, y: SteelVal, color: String, size: SteelVal) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let result = draw_text(&img, &TextParams {
        text, x: to_f64(&x)? as i32, y: to_f64(&y)? as i32,
        color, font_size: to_u32(&size)?,
        align: "left".to_string(),
        background_color: None,
        max_width: None,
    });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_draw_rectangle(img_id: String, x: SteelVal, y: SteelVal, width: SteelVal, height: SteelVal, color: String, filled: bool) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let result = draw_rectangle(&img, &RectangleParams {
        x: to_f64(&x)? as i32, y: to_f64(&y)? as i32,
        width: to_u32(&width)?, height: to_u32(&height)?,
        color, filled,
        border_radius: 0,
        stroke_width: 0,
        stroke_color: None,
    });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_draw_circle(img_id: String, x: isize, y: isize, radius: isize, color: String, filled: bool) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let result = draw_circle(&img, &CircleMaskParams {
        x: x as i32, y: y as i32,
        radius: radius as u32,
        color, filled,
        stroke_width: 0,
        stroke_color: None,
    });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_draw_line(img_id: String, x1: SteelVal, y1: SteelVal, x2: SteelVal, y2: SteelVal, color: String, thickness: SteelVal) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let result = draw_line(&img, &LineParams {
        x1: to_f64(&x1)? as i32, y1: to_f64(&y1)? as i32,
        x2: to_f64(&x2)? as i32, y2: to_f64(&y2)? as i32,
        color, thickness: to_u32(&thickness)?,
    });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_overlay(base_id: String, overlay_id: String, x: SteelVal, y: SteelVal, opacity: f64) -> Result<String, String> {
    let base = get_img(&base_id)?;
    let overlay_img = get_img(&overlay_id)?;
    let result = overlay(&base, &overlay_img, &OverlayParams {
        base_image: String::new(),
        overlay_image: String::new(),
        x: to_f64(&x)? as i32, y: to_f64(&y)? as i32,
        opacity: Some(opacity as f32),
        blend_mode: None,
    });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_apply_mask(img_id: String, mask_id: String) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let mask = get_img(&mask_id)?;
    let result = apply_mask(&img, &mask);
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_drop_shadow(img_id: String, offset_x: SteelVal, offset_y: SteelVal, blur_radius: SteelVal, color: String) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let result = drop_shadow(&img, &ShadowParams {
        offset_x: to_f64(&offset_x)? as i32,
        offset_y: to_f64(&offset_y)? as i32,
        blur_radius: to_u32(&blur_radius)?,
        color,
    });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_liquify(img_id: String, mode: String, center_x: f64, center_y: f64, radius: f64, strength: f64) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let result = liquify(&img, &LiquifyParams { mode, center_x: center_x as f32, center_y: center_y as f32, radius: radius as f32, strength: strength as f32 });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_warp(img_id: String, mode: String, amplitude: f64, frequency: f64) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let result = warp(&img, &WarpParams { mode, amplitude: amplitude as f32, frequency: frequency as f32 });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_glitch(img_id: String, mode: String, intensity: f64, seed: isize) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let result = glitch(&img, &GlitchParams { mode, intensity: intensity as f32, seed: seed as u32 });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_chromatic_aberration(img_id: String, shift: f64, auto: bool) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let result = chromatic_aberration(&img, &ChromaticAberrationParams { shift: shift as f32, auto });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_pixel_sort(img_id: String, axis: String, threshold_low: f64, threshold_high: f64) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let result = pixel_sort(&img, &PixelSortParams { axis, threshold_low: threshold_low as f32, threshold_high: threshold_high as f32 });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_scanlines(img_id: String, spacing: isize, opacity: f64, thickness: isize) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let result = scanlines(&img, &ScanlinesParams { spacing: spacing as u32, opacity: opacity as f32, thickness: thickness as u32 });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_halftone(img_id: String, dot_size: isize, angle: f64, mode: String) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let result = halftone(&img, &HalftoneParams { dot_size: dot_size as u32, angle: angle as f32, mode });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_posterize(img_id: String, levels: isize) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let result = posterize(&img, &PosterizeParams { levels: levels as u8 });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_noise(img_id: String, amount: f64, noise_type: String, seed: isize) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let result = noise(&img, &NoiseParams { amount: amount as f32, noise_type, seed: seed as u32 });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_kaleidoscope(img_id: String, segments: isize, offset: f64) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let result = kaleidoscope(&img, &KaleidoscopeParams { segments: segments as u32, offset: offset as f32 });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_emboss(img_id: String, strength: f64) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let result = emboss(&img, &EmbossParams { strength: strength as f32 });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_edge_detect(img_id: String, threshold: isize, invert: bool) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let result = edge_detect(&img, &EdgeDetectParams { threshold: threshold as u8, invert });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_solarize(img_id: String, threshold: isize) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let result = solarize(&img, &SolarizeParams { threshold: threshold as u8 });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

// ─── Generators (no input image) ───

fn steel_gradient(width: SteelVal, height: SteelVal, start_color: String, end_color: String, angle: SteelVal) -> Result<String, String> {
    let result = create_gradient(&GradientParams {
        width: to_u32(&width)?, height: to_u32(&height)?,
        start_color, end_color,
        angle: to_f64(&angle)? as f32,
    });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_canvas(width: isize, height: isize, color: String) -> Result<String, String> {
    let rgba = hex_to_rgba(&color).map_err(|e| e.to_string())?;
    let img = DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(width as u32, height as u32, rgba));
    let id = new_id();
    register_img(&id, img);
    Ok(id)
}

fn steel_frame(width: isize, height: isize, frame_width: isize, color: String) -> Result<String, String> {
    let result = create_frame(&FrameParams {
        width: width as u32, height: height as u32,
        frame_width: frame_width as u32,
        frame_color: color,
        inner_color: None,
    });
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

// ─── Utility ───

fn steel_image_dimensions(img_id: String) -> Result<Vec<steel::SteelVal>, String> {
    let img = get_img(&img_id)?;
    let (w, h) = img.dimensions();
    Ok(vec![
        steel::SteelVal::IntV(w as isize),
        steel::SteelVal::IntV(h as isize),
    ])
}

fn steel_comparison(original_id: String, edited_id: String, orientation: String, label1: Option<String>, label2: Option<String>) -> Result<String, String> {
    let original = get_img(&original_id)?;
    let edited = get_img(&edited_id)?;
    let params = ComparisonParams {
        original_image: String::new(),
        edited_image: String::new(),
        orientation,
        gap: 20,
        label_original: label1,
        label_edited: label2,
        label_size: 24,
        label_color: "#FFFFFF".to_string(),
        background_color: "#1a1a2e".to_string(),
    };
    let result = create_comparison(&original, &edited, &params);
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_steps_grid(images: Vec<String>, labels: Option<Vec<String>>, width: isize, height: isize) -> Result<String, String> {
    let mut imgs = Vec::new();
    for img_id in &images {
        imgs.push(get_img(img_id)?);
    }
    let params = StepsGridParams {
        images: Vec::new(),
        labels,
        width: width as u32,
        height: height as u32,
        gap: 10,
        max_columns: 4,
        label_size: 20,
        label_color: "#FFFFFF".to_string(),
        background_color: "#1a1a2e".to_string(),
    };
    let result = create_steps_grid(&imgs, &params);
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_diff(original_id: String, edited_id: String) -> Result<String, String> {
    let original = get_img(&original_id)?;
    let edited = get_img(&edited_id)?;
    let params = DiffParams {
        original_image: String::new(),
        edited_image: String::new(),
        threshold: 30,
    };
    let result = create_diff(&original, &edited, &params);
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

// ─── Async Bridge Functions ───
//
// Steel's VM is synchronous, but we need to call async Replicate / fal.ai APIs.
// We solve this by using tokio::spawn inside a Tokio task, then blocking on an
// std::sync::mpsc channel.
//
fn steel_seg_everything(img_id: String) -> Result<Vec<String>, String> {
    let img = get_img(&img_id)?;
    let b64 = save_to_base64(&img, ImageFormat::Png).map_err(|e| e.to_string())?;
    let params = SegmentImageParams {
        image_b64: b64,
        api_key: None,
        model: "meta/sam-2:fe97b453a6455861e3bac769b441ca1f1086110da7466dbb65cf1eecfd60dc83".to_string(),
    };

    let (tx, rx) = std::sync::mpsc::channel();
    tokio::spawn(async move {
        let result = segment_images_to_b64(&params).await;
        let _ = tx.send(result);
    });

    let (combined_b64, masks_b64) = match rx.recv() {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => return Err(format!("Segmentation error: {}", e)),
        Err(e) => return Err(format!("Channel error: {}", e)),
    };

    // Register combined mask
    let mut ids: Vec<String> = Vec::new();
    let combined = load_from_base64(&combined_b64).map_err(|e| e.to_string())?;
    let combined_id = new_id();
    register_img(&combined_id, combined);
    ids.push(combined_id);

    // Register each individual mask
    for mask_b64 in masks_b64 {
        let mask = load_from_base64(&mask_b64).map_err(|e| e.to_string())?;
        let mask_id = new_id();
        register_img(&mask_id, mask);
        ids.push(mask_id);
    }

    Ok(ids)
}

fn steel_remove_background(img_id: String) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let b64 = save_to_base64(&img, ImageFormat::Png).map_err(|e| e.to_string())?;
    let params = RemoveBackgroundParams {
        image_b64: b64,
        api_key: None,
    };

    let (tx, rx) = std::sync::mpsc::channel();
    tokio::spawn(async move {
        let result = remove_background_to_b64(&params).await;
        let _ = tx.send(result);
    });

    let b64 = match rx.recv() {
        Ok(Ok(b64)) => b64,
        Ok(Err(e)) => return Err(format!("Remove background error: {}", e)),
        Err(e) => return Err(format!("Channel error: {}", e)),
    };

    let result = load_from_base64(&b64).map_err(|e| e.to_string())?;
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

fn steel_fal_edit_image(img_id: String, prompt: String, mask_id: Option<String>) -> Result<String, String> {
    let img = get_img(&img_id)?;
    let b64 = save_to_base64(&img, ImageFormat::Png).map_err(|e| e.to_string())?;
    let mask_b64 = mask_id.and_then(|mid| {
        get_img(&mid).ok().and_then(|m| save_to_base64(&m, ImageFormat::Png).ok())
    });
    let params = crate::operations::generation::FalEditParams {
        prompt,
        image_b64: b64,
        mask_b64,
        quality: "high".to_string(),
        num_images: 1,
        output_format: "png".to_string(),
        image_size: "auto".to_string(),
        api_key: None,
    };

    let (tx, rx) = std::sync::mpsc::channel();
    tokio::spawn(async move {
        let result = crate::operations::generation::fal_edit_image_to_b64(&params).await;
        let _ = tx.send(result);
    });

    let b64 = match rx.recv() {
        Ok(Ok(b64)) => b64,
        Ok(Err(e)) => return Err(format!("GPT Image 2 edit error: {}", e)),
        Err(e) => return Err(format!("Channel error: {}", e)),
    };

    let result = load_from_base64(&b64).map_err(|e| e.to_string())?;
    let id = new_id();
    register_img(&id, result);
    Ok(id)
}

// ─── Engine Setup & Execution ───

pub fn run_scheme_script(script: &str, input_image_b64: Option<&str>) -> Result<(String, String), String> {
    // Clear registry from previous runs
    with_registry(|reg| reg.clear());

    let mut engine = Engine::new();

    // Register all image functions
    // NOTE: must be done BEFORE loading lib.scm so the functions are available
    macro_rules! reg {
        ($name:literal, $func:expr) => {
            engine.register_fn($name, $func);
        };
    }

    reg!("image-crop", steel_crop);
    reg!("image-crop-center", steel_crop_center);
    reg!("image-smart-crop", steel_smart_crop);
    reg!("image-resize", steel_resize);
    reg!("image-thumbnail", steel_thumbnail);
    reg!("image-rotate", steel_rotate);
    reg!("image-flip", steel_flip);
    reg!("image-blur", steel_blur);
    reg!("image-brightness", steel_brightness);
    reg!("image-contrast", steel_contrast);
    reg!("image-grayscale", steel_grayscale);
    reg!("image-tint", steel_tint);
    reg!("image-vignette", steel_vignette);
    reg!("image-duotone", steel_duotone);
    reg!("image-draw-text", steel_draw_text);
    reg!("image-draw-rectangle", steel_draw_rectangle);
    reg!("image-draw-circle", steel_draw_circle);
    reg!("image-draw-line", steel_draw_line);
    reg!("image-overlay", steel_overlay);
    reg!("image-apply-mask", steel_apply_mask);
    reg!("image-drop-shadow", steel_drop_shadow);
    reg!("image-liquify", steel_liquify);
    reg!("image-warp", steel_warp);
    reg!("image-glitch", steel_glitch);
    reg!("image-chromatic-aberration", steel_chromatic_aberration);
    reg!("image-pixel-sort", steel_pixel_sort);
    reg!("image-scanlines", steel_scanlines);
    reg!("image-halftone", steel_halftone);
    reg!("image-posterize", steel_posterize);
    reg!("image-noise", steel_noise);
    reg!("image-kaleidoscope", steel_kaleidoscope);
    reg!("image-emboss", steel_emboss);
    reg!("image-edge-detect", steel_edge_detect);
    reg!("image-solarize", steel_solarize);
    reg!("image-gradient", steel_gradient);
    reg!("image-canvas", steel_canvas);
    reg!("image-frame", steel_frame);
    reg!("image-dimensions", steel_image_dimensions);
    reg!("image-comparison", steel_comparison);
    reg!("image-steps-grid", steel_steps_grid);
    reg!("image-diff", steel_diff);
    reg!("image-segment-everything", steel_seg_everything);
    reg!("image-remove-background", steel_remove_background);
    reg!("image-fal-edit", steel_fal_edit_image);

    // Register input image
    if let Some(b64) = input_image_b64 {
        let img = b64_to_img(b64)?;
        register_img("input", img);
        engine.register_value("input", SteelVal::StringV("input".into()));
    }

    // Load standard library (must be after fn registration)
    const LIB_SCM: &str = include_str!("../../examples/lib.scm");
    if let Err(e) = engine.run(LIB_SCM.to_string()) {
        eprintln!("Warning: lib.scm loading produced error: {}", e);
    }

    // Run the script - needs owned string for steel-core's lifetime requirements
    let script_owned = script.to_string();
    engine.run(script_owned).map_err(|e| format!("Steel execution error: {}", e))?;

    // Extract the result - look for a variable named `result` or `output`
    // or check if the last expression evaluated to an image ID string
    let result_id: Option<String> = engine.extract("result")
        .or_else(|_| engine.extract("output"))
        .ok();

    let id = match result_id {
        Some(id) => id,
        None => {
            // Try to find any image in the registry
            with_registry(|reg| {
                reg.keys().next().cloned()
                    .ok_or_else(|| "No image produced by script. Assign the final image ID to a variable named `result` or `output`.".to_string())
            })?
        }
    };

    // Also provide a base64 version as a variable for Scheme to access
    let img = get_img(&id)?;
    let b64 = img_to_b64(&img)?;

    Ok((id, b64))
}
