use crate::operations::generation::*;
use crate::operations::preview::*;
use crate::operations::segmentation::*;
use crate::operations::*;
use image::{DynamicImage, GenericImageView, ImageFormat};
use rmcp::{
    ErrorData, ServerHandler, ServiceExt,
    model::*,
    service::RequestContext,
    transport::stdio,
};
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(Clone)]
pub struct ImageDesignServer;

impl ImageDesignServer {
    pub fn new() -> Self {
        Self
    }

    fn b64_to_img(b64: &str) -> anyhow::Result<DynamicImage> {
        load_from_base64(b64)
    }

    fn img_to_b64(img: &DynamicImage) -> anyhow::Result<String> {
        save_to_base64(img, ImageFormat::Png)
    }

    fn make_image_result(img: &DynamicImage) -> Result<CallToolResult, String> {
        let b64 = Self::img_to_b64(img).map_err(|e| e.to_string())?;
        Ok(CallToolResult::success(vec![
            RawContent::image(b64, "image/png").no_annotation(),
        ]))
    }

    fn make_image_with_preview(img: &DynamicImage) -> Result<CallToolResult, String> {
        let b64 = Self::img_to_b64(img).map_err(|e| e.to_string())?;
        let preview = generate_preview_result(img).map_err(|e| e.to_string())?;
        Ok(CallToolResult::success(vec![
            RawContent::image(b64, "image/png").no_annotation(),
            RawContent::text(format!(
                "Done. Full: {}x{}. Preview: {}x{}",
                img.width(), img.height(), preview.width, preview.height
            )).no_annotation(),
            RawContent::image(preview.data, &preview.mime_type).no_annotation(),
        ]))
    }
}

impl ServerHandler for ImageDesignServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .build(),
        )
        .with_server_info(Implementation::from_build_env())
        .with_instructions(
            "MCP Image Design Server: create images, edit them, compare versions, generate with AI.".to_string(),
        )
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<rmcp::RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let name = request.name.as_ref();
        let args = Value::Object(request.arguments.unwrap_or_default());

        match name {
            "create_image" => self.handle_create_image(args.clone()).map_err(|e|
                ErrorData::internal_error(format!("[create_image] {}", e), None)),
            "edit_image" => self.handle_edit_image(args.clone()).map_err(|e|
                ErrorData::internal_error(format!("[edit_image] {}", e), None)),
            "compose" => self.handle_compose(args.clone()).map_err(|e|
                ErrorData::internal_error(format!("[compose] {}", e), None)),
            "diff" => self.handle_diff(args).await.map_err(|e|
                ErrorData::internal_error(format!("[diff] {}", e), None)),
            "generate_image" => self.handle_generate_image(args).await.map_err(|e|
                ErrorData::internal_error(format!("[generate_image] {}", e), None)),
            "fill_with_generation" => self.handle_fill_with_generation(args).await.map_err(|e|
                ErrorData::internal_error(format!("[fill_with_generation] {}", e), None)),
            "edit_image_ai" => self.handle_edit_image_ai(args).await.map_err(|e|
                ErrorData::internal_error(format!("[edit_image_ai] {}", e), None)),
            "segment_image" => self.handle_segment_image(args).await.map_err(|e|
                ErrorData::internal_error(format!("[segment_image] {}", e), None)),
            "remove_background" => self.handle_remove_background(args).await.map_err(|e|
                ErrorData::internal_error(format!("[remove_background] {}", e), None)),
            "run_scheme" => self.handle_run_scheme(args.clone()).map_err(|e|
                ErrorData::internal_error(format!("[run_scheme] {}", e), None)),
            _ => Err(ErrorData::invalid_params(
                format!("Unknown tool '{}'. Available tools: create_image, edit_image, compose, diff, generate_image, fill_with_generation, edit_image_ai, segment_image, remove_background, run_scheme", name),
                None,
            )),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<rmcp::RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let tools = vec![
            Tool::new(
                "create_image",
                "Create a new image. Types: gradient, canvas, frame, social_post, youtube_thumbnail.",
                Arc::new(json!({
                    "type": "object",
                    "properties": {
                        "type": { "type": "string", "enum": ["gradient", "canvas", "frame", "social_post", "youtube_thumbnail"] },
                        "width": { "type": "integer" },
                        "height": { "type": "integer" },
                        "startColor": { "type": "string" },
                        "endColor": { "type": "string" },
                        "angle": { "type": "number" },
                        "color": { "type": "string" },
                        "frameWidth": { "type": "integer" },
                        "frameColor": { "type": "string" },
                        "text": { "type": "string" },
                        "textColor": { "type": "string" },
                        "fontSize": { "type": "integer" },
                        "textX": { "type": "integer" },
                        "textY": { "type": "integer" },
                        "gradientStart": { "type": "string" },
                        "gradientEnd": { "type": "string" },
                        "addVignette": { "type": "boolean" },
                        "blurBg": { "type": "boolean" },
                        "image": { "type": "string", "description": "Base64 background image" }
                    },
                    "required": ["type", "width", "height"]
                }).as_object().unwrap().clone()),
            ),
            Tool::new(
                "edit_image",
                "Apply edits to an image. Each op has 'op' plus params.",
                Arc::new(json!({
                    "type": "object",
                    "properties": {
                        "image": { "type": "string" },
                        "operations": { "type": "array", "description": "Array of operation objects with 'op' field" }
                    },
                    "required": ["image", "operations"]
                }).as_object().unwrap().clone()),
            ),
            Tool::new(
                "compose",
                "Combine images: overlay, collage, shape_mask.",
                Arc::new(json!({
                    "type": "object",
                    "properties": {
                        "mode": { "type": "string", "enum": ["overlay", "collage", "shape_mask"] },
                        "baseImage": { "type": "string" },
                        "overlayImage": { "type": "string" },
                        "x": { "type": "integer" },
                        "y": { "type": "integer" },
                        "opacity": { "type": "number" },
                        "blendMode": { "type": "string", "enum": ["normal","multiply","screen","overlay","darken","lighten"] },
                        "images": { "type": "array", "items": { "type": "string" } },
                        "layout": { "type": "string", "enum": ["grid","horizontal","vertical"] },
                        "width": { "type": "integer" },
                        "height": { "type": "integer" },
                        "gap": { "type": "integer", "default": 10 },
                        "shape": { "type": "string", "enum": ["circle","ellipse","rounded_rect","rect"] },
                        "maskX": { "type": "integer" },
                        "maskY": { "type": "integer" },
                        "maskWidth": { "type": "integer" },
                        "maskHeight": { "type": "integer" },
                        "feather": { "type": "boolean" }
                    },
                    "required": ["mode"]
                }).as_object().unwrap().clone()),
            ),
            Tool::new(
                "diff",
                "Compare images. Modes: side_by_side, grid, pixel_diff. Returns compressed preview.",
                Arc::new(json!({
                    "type": "object",
                    "properties": {
                        "mode": { "type": "string", "enum": ["side_by_side", "grid", "pixel_diff"] },
                        "originalImage": { "type": "string" },
                        "editedImage": { "type": "string" },
                        "images": { "type": "array", "items": { "type": "string" } },
                        "labels": { "type": "array", "items": { "type": "string" } },
                        "width": { "type": "integer" },
                        "height": { "type": "integer" },
                        "orientation": { "type": "string", "enum": ["horizontal","vertical"], "default": "horizontal" },
                        "gap": { "type": "integer", "default": 20 },
                        "maxColumns": { "type": "integer", "default": 4 },
                        "threshold": { "type": "integer", "default": 30 },
                        "backgroundColor": { "type": "string", "default": "#1a1a2e" },
                        "labelColor": { "type": "string", "default": "#FFFFFF" }
                    },
                    "required": ["mode"]
                }).as_object().unwrap().clone()),
            ),
            Tool::new(
                "generate_image",
                "Generate an image using OpenAI DALL-E. Requires OPENAI_API_KEY env var.",
                Arc::new(json!({
                    "type": "object",
                    "properties": {
                        "prompt": { "type": "string" },
                        "width": { "type": "integer", "default": 1024 },
                        "height": { "type": "integer", "default": 1024 },
                        "model": { "type": "string", "enum": ["dall-e-3", "dall-e-2"], "default": "dall-e-3" },
                        "quality": { "type": "string", "enum": ["standard", "hd"], "default": "standard" },
                        "style": { "type": "string", "enum": ["vivid", "natural"], "default": "vivid" }
                    },
                    "required": ["prompt"]
                }).as_object().unwrap().clone()),
            ),
            Tool::new(
                "edit_image_ai",
                "Edit an image using GPT Image 2 via fal.ai. Supports mask-based inpainting and natural language editing. Requires FAL_KEY.",
                Arc::new(json!({
                    "type": "object",
                    "properties": {
                        "image": { "type": "string", "description": "Base64 image to edit" },
                        "prompt": { "type": "string", "description": "Description of the desired edit" },
                        "mask": { "type": "string", "description": "Optional base64 mask (white = edit area)" },
                        "imageSize": { "type": "string", "enum": ["auto","square_hd","square","portrait_4_3","portrait_16_9","landscape_4_3","landscape_16_9"], "default": "auto" },
                        "quality": { "type": "string", "enum": ["auto","low","medium","high"], "default": "high" },
                        "outputFormat": { "type": "string", "enum": ["png","jpeg","webp"], "default": "png" },
                        "numImages": { "type": "integer", "minimum": 1, "maximum": 4, "default": 1 }
                    },
                    "required": ["image", "prompt"]
                }).as_object().unwrap().clone()),
            ),
            Tool::new(
                "fill_with_generation",
                "Fill a masked region with AI-generated content. Requires OPENAI_API_KEY.",
                Arc::new(json!({
                    "type": "object",
                    "properties": {
                        "prompt": { "type": "string" },
                        "image": { "type": "string", "description": "Original image (base64)" },
                        "mask": { "type": "string", "description": "Mask image (base64)" },
                        "model": { "type": "string", "enum": ["gpt-image-1", "dall-e-2"], "default": "gpt-image-1" }
                    },
                    "required": ["prompt", "image", "mask"]
                }).as_object().unwrap().clone()),
            ),
            Tool::new(
                "segment_image",
                "Segment an image into objects using Meta SAM-2 via Replicate. Returns combined mask + individual masks.",
                Arc::new(json!({
                    "type": "object",
                    "properties": {
                        "image": { "type": "string", "description": "Base64 image to segment" },
                        "model": { "type": "string", "enum": ["meta/sam-2", "meta/sam-2:fe97b453a6455861e3bac769b441ca1f1086110da7466dbb65cf1eecfd60dc83"], "default": "meta/sam-2:fe97b453a6455861e3bac769b441ca1f1086110da7466dbb65cf1eecfd60dc83", "description": "Replicate model identifier" }
                    },
                    "required": ["image"]
                }).as_object().unwrap().clone()),
            ),
            Tool::new(
                "remove_background",
                "Remove the background from an image via Replicate remove-bg model.",
                Arc::new(json!({
                    "type": "object",
                    "properties": {
                        "image": { "type": "string", "description": "Base64 image with background to remove" }
                    },
                    "required": ["image"]
                }).as_object().unwrap().clone()),
            ),
            Tool::new(
                "run_scheme",
                "Execute Steel Scheme code for complex multi-step image processing.",
                Arc::new(json!({
                    "type": "object",
                    "properties": {
                        "script": { "type": "string" },
                        "inputImage": { "type": "string" }
                    },
                    "required": ["script"]
                }).as_object().unwrap().clone()),
            ),
        ];
        Ok(ListToolsResult {
            tools,
            next_cursor: None,
            meta: None,
        })
    }
}

impl ImageDesignServer {
    fn handle_create_image(&self, args: Value) -> Result<CallToolResult, String> {
        let mode = args["type"].as_str().unwrap_or("canvas");
        let width = args["width"].as_u64().unwrap_or(400) as u32;
        let height = args["height"].as_u64().unwrap_or(400) as u32;

        let img = match mode {
            "gradient" => {
                let start = args["startColor"].as_str().unwrap_or("#667eea");
                let end = args["endColor"].as_str().unwrap_or("#764ba2");
                let angle = args["angle"].as_f64().unwrap_or(135.0) as f32;
                create_gradient(&GradientParams {
                    width, height,
                    start_color: start.to_string(),
                    end_color: end.to_string(),
                    angle,
                })
            }
            "canvas" => {
                let color = args["color"].as_str().unwrap_or("#FFFFFF");
                let rgba = hex_to_rgba(color).map_err(|e| e.to_string())?;
                DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(width, height, rgba))
            }
            "frame" => {
                let fw = args["frameWidth"].as_u64().unwrap_or(20) as u32;
                let fc = args["frameColor"].as_str().unwrap_or("#333333");
                create_frame(&FrameParams {
                    width, height,
                    frame_width: fw,
                    frame_color: fc.to_string(),
                    inner_color: None,
                })
            }
            "social_post" => {
                let mut canvas = if let Some(img_b64) = args.get("image").and_then(|v| v.as_str()) {
                    let img = load_from_base64(img_b64).map_err(|e| e.to_string())?;
                    let resized = img.resize(width, height, image::imageops::FilterType::Lanczos3);
                    if args.get("blurBg").and_then(|v| v.as_bool()).unwrap_or(false) {
                        resized.blur(5.0)
                    } else {
                        resized
                    }
                } else {
                    let gs = args["gradientStart"].as_str().unwrap_or("#667eea");
                    let ge = args["gradientEnd"].as_str().unwrap_or("#764ba2");
                    create_gradient(&GradientParams {
                        width, height,
                        start_color: gs.to_string(),
                        end_color: ge.to_string(),
                        angle: 135.0,
                    })
                };
                if args.get("addVignette").and_then(|v| v.as_bool()).unwrap_or(false) {
                    canvas = vignette(&canvas, &VignetteParams {
                        strength: 0.4,
                        radius: 1.0,
                        color: None,
                    });
                }
                let text = args["text"].as_str().unwrap_or("");
                let tc = args["textColor"].as_str().unwrap_or("#FFFFFF");
                let fs = args["fontSize"].as_u64().unwrap_or(48) as u32;
                let tx = args["textX"].as_i64().unwrap_or(50) as i32;
                let ty = args["textY"].as_i64().unwrap_or(height as i64 / 2) as i32;
                canvas = draw_text(&canvas, &TextParams {
                    text: text.to_string(),
                    x: tx, y: ty,
                    color: tc.to_string(),
                    font_size: fs,
                    align: "center".to_string(),
                    background_color: None,
                    max_width: Some(width - 100),
                });
                canvas
            }
            "youtube_thumbnail" => {
                let mut canvas = if let Some(img_b64) = args.get("image").and_then(|v| v.as_str()) {
                    let img = load_from_base64(img_b64).map_err(|e| e.to_string())?;
                    img.resize(width.max(1280), height.max(720), image::imageops::FilterType::Lanczos3)
                } else {
                    create_gradient(&GradientParams {
                        width: width.max(1280), height: height.max(720),
                        start_color: "#ff006e".to_string(),
                        end_color: "#8338ec".to_string(),
                        angle: 45.0,
                    })
                };
                canvas = vignette(&canvas, &VignetteParams {
                    strength: 0.4, radius: 1.0, color: Some("#000000".to_string()),
                });
                let text = args["text"].as_str().unwrap_or("CLICKBAIT TITLE");
                let tc = args["textColor"].as_str().unwrap_or("#ffffff");
                let fs = args["fontSize"].as_u64().unwrap_or(96) as u32;
                let tx = args["textX"].as_i64().unwrap_or(100) as i32;
                let ty = args["textY"].as_i64().unwrap_or(500) as i32;
                canvas = draw_text(&canvas, &TextParams {
                    text: text.to_string(), x: tx, y: ty,
                    color: tc.to_string(), font_size: fs,
                    align: "left".to_string(), background_color: None,
                    max_width: Some(canvas.width() - 200),
                });
                canvas
            }
            _ => return Err(format!("Unknown create_image type: {}", mode)),
        };

        Self::make_image_with_preview(&img)
    }

    fn handle_edit_image(&self, args: Value) -> Result<CallToolResult, String> {
        let mut img = Self::b64_to_img(
            args["image"].as_str().ok_or("Missing image")?
        ).map_err(|e| e.to_string())?;
        let ops = args["operations"]
            .as_array()
            .ok_or("operations must be an array")?;

        for op_val in ops {
            let op = op_val["op"]
                .as_str()
                .ok_or("Each operation needs an 'op' field")?;
            img = match op {
                "blur" => blur(&img, &BlurParams { sigma: op_val["sigma"].as_f64().unwrap_or(1.0) as f32 }),
                "brightness" => brightness(&img, &BrightnessParams { value: op_val["value"].as_f64().unwrap_or(0.0) as f32 }),
                "contrast" => filters::contrast(&img, &ContrastParams { value: op_val["value"].as_f64().unwrap_or(0.0) as f32 }),
                "grayscale" => grayscale(&img, &GrayscaleParams { mode: "luma".to_string() }),
                "tint" => tint(&img, &TintParams { color: op_val["color"].as_str().unwrap_or("#808080").to_string(), amount: op_val["amount"].as_f64().unwrap_or(0.5) as f32 }),
                "vignette" => vignette(&img, &VignetteParams { strength: op_val["strength"].as_f64().unwrap_or(0.5) as f32, radius: op_val["radius"].as_f64().unwrap_or(1.0) as f32, color: op_val["color"].as_str().map(|s| s.to_string()) }),
                "duotone" => duotone(&img, &DuotoneParams { shadow_color: op_val["shadowColor"].as_str().unwrap_or("#000000").to_string(), highlight_color: op_val["highlightColor"].as_str().unwrap_or("#FFFFFF").to_string() }),
                "draw_text" => draw_text(&img, &TextParams { text: op_val["text"].as_str().unwrap_or("").to_string(), x: op_val["x"].as_i64().unwrap_or(0) as i32, y: op_val["y"].as_i64().unwrap_or(0) as i32, color: op_val["color"].as_str().unwrap_or("#FFFFFF").to_string(), font_size: op_val["fontSize"].as_u64().unwrap_or(24) as u32, align: "left".to_string(), background_color: None, max_width: None }),
                "drop_shadow" => drop_shadow(&img, &ShadowParams { offset_x: op_val["offsetX"].as_i64().unwrap_or(5) as i32, offset_y: op_val["offsetY"].as_i64().unwrap_or(5) as i32, blur_radius: op_val["blurRadius"].as_u64().unwrap_or(10) as u32, color: op_val["color"].as_str().unwrap_or("#00000080").to_string() }),
                "glitch" => glitch(&img, &GlitchParams { mode: op_val["mode"].as_str().unwrap_or("slice_shift").to_string(), intensity: op_val["intensity"].as_f64().unwrap_or(0.5) as f32, seed: op_val["seed"].as_u64().unwrap_or(42) as u32 }),
                "chromatic_aberration" => chromatic_aberration(&img, &ChromaticAberrationParams { shift: op_val["shift"].as_f64().unwrap_or(5.0) as f32, auto: op_val["auto"].as_bool().unwrap_or(false) }),
                "halftone" => halftone(&img, &HalftoneParams { dot_size: op_val["dotSize"].as_u64().unwrap_or(8) as u32, angle: op_val["angle"].as_f64().unwrap_or(45.0) as f32, mode: op_val["mode"].as_str().unwrap_or("color").to_string() }),
                "noise" => noise(&img, &NoiseParams { amount: op_val["amount"].as_f64().unwrap_or(0.1) as f32, noise_type: op_val["noiseType"].as_str().unwrap_or("grain").to_string(), seed: op_val["seed"].as_u64().unwrap_or(42) as u32 }),
                "edge_detect" => edge_detect(&img, &EdgeDetectParams { threshold: op_val["threshold"].as_u64().unwrap_or(30) as u8, invert: op_val["invert"].as_bool().unwrap_or(false) }),
                "resize" => {
                    let mode = match op_val["mode"].as_str().unwrap_or("exact") {
                        "fit" => ResizeMode::Fit, "fill" => ResizeMode::Fill,
                        "scale" => ResizeMode::Scale, _ => ResizeMode::Exact,
                    };
                    resize(&img, &ResizeParams { width: op_val["width"].as_u64().map(|u| u as u32), height: op_val["height"].as_u64().map(|u| u as u32), mode, filter: "lanczos3".to_string() })
                }
                "crop" => crop(&img, &CropParams { x: op_val["x"].as_u64().unwrap_or(0) as u32, y: op_val["y"].as_u64().unwrap_or(0) as u32, width: op_val["width"].as_u64().unwrap_or(100) as u32, height: op_val["height"].as_u64().unwrap_or(100) as u32 }),
                "rotate" => rotate(&img, &RotateParams { degrees: op_val["degrees"].as_f64().unwrap_or(0.0) as f32, background_color: op_val["backgroundColor"].as_str().map(|s| s.to_string()) }),
                "flip" => flip(&img, &FlipParams { horizontal: op_val["horizontal"].as_bool().unwrap_or(false), vertical: op_val["vertical"].as_bool().unwrap_or(false) }),
                "crop_center" => crop_from_center(&img, &CropFromCenterParams { width: op_val["width"].as_u64().unwrap_or(100) as u32, height: op_val["height"].as_u64().unwrap_or(100) as u32 }),
                "liquify" => liquify(&img, &LiquifyParams { mode: op_val["mode"].as_str().unwrap_or("bulge").to_string(), center_x: op_val["centerX"].as_f64().unwrap_or(0.5) as f32, center_y: op_val["centerY"].as_f64().unwrap_or(0.5) as f32, radius: op_val["radius"].as_f64().unwrap_or(0.3) as f32, strength: op_val["strength"].as_f64().unwrap_or(0.5) as f32 }),
                _ => return Err(format!("Unknown edit operation: {}", op)),
            };
        }

        Self::make_image_with_preview(&img)
    }

    fn handle_compose(&self, args: Value) -> Result<CallToolResult, String> {
        let mode = args["mode"].as_str().unwrap_or("overlay");
        let img = match mode {
            "overlay" => {
                let base = Self::b64_to_img(args["baseImage"].as_str().ok_or("Missing baseImage")?).map_err(|e| e.to_string())?;
                let overlay_img = Self::b64_to_img(args["overlayImage"].as_str().ok_or("Missing overlayImage")?).map_err(|e| e.to_string())?;
                overlay(&base, &overlay_img, &OverlayParams {
                    base_image: String::new(), overlay_image: String::new(),
                    x: args["x"].as_i64().unwrap_or(0) as i32,
                    y: args["y"].as_i64().unwrap_or(0) as i32,
                    opacity: args["opacity"].as_f64().map(|f| f as f32),
                    blend_mode: args["blendMode"].as_str().map(|s| s.to_string()),
                })
            }
            "collage" => {
                let images_b64: Vec<String> = serde_json::from_value(args["images"].clone()).map_err(|e| e.to_string())?;
                let mut images = Vec::new();
                for b64 in &images_b64 { images.push(load_from_base64(b64).map_err(|e| e.to_string())?); }
                let params = CollageParams {
                    images: Vec::new(),
                    layout: args["layout"].as_str().unwrap_or("grid").to_string(),
                    width: args["width"].as_u64().unwrap_or(800) as u32,
                    height: args["height"].as_u64().unwrap_or(600) as u32,
                    gap: args["gap"].as_u64().unwrap_or(10) as u32,
                };
                create_collage(&images, &params)
            }
            "shape_mask" => {
                let img = Self::b64_to_img(args["baseImage"].as_str().ok_or("Missing baseImage")?).map_err(|e| e.to_string())?;
                let mask = create_shape_mask(&img, &ShapeMaskParams {
                    shape: args["shape"].as_str().unwrap_or("circle").to_string(),
                    x: args["maskX"].as_i64().unwrap_or(0) as i32,
                    y: args["maskY"].as_i64().unwrap_or(0) as i32,
                    width: args["maskWidth"].as_u64().unwrap_or(100) as u32,
                    height: args["maskHeight"].as_u64().unwrap_or(100) as u32,
                    feather: args["feather"].as_bool().unwrap_or(false),
                    feather_radius: 10,
                });
                apply_mask(&img, &mask)
            }
            _ => return Err(format!("Unknown compose mode: {}", mode)),
        };

        Self::make_image_with_preview(&img)
    }

    async fn handle_diff(&self, args: Value) -> Result<CallToolResult, String> {
        let mode = args["mode"].as_str().unwrap_or("side_by_side");
        let bg = args["backgroundColor"].as_str().unwrap_or("#1a1a2e").to_string();
        let label_color = args["labelColor"].as_str().unwrap_or("#FFFFFF").to_string();

        let (img, preview) = match mode {
            "side_by_side" => {
                let original = Self::b64_to_img(args["originalImage"].as_str().ok_or("Missing originalImage")?).map_err(|e| e.to_string())?;
                let edited = Self::b64_to_img(args["editedImage"].as_str().ok_or("Missing editedImage")?).map_err(|e| e.to_string())?;
                let params = ComparisonParams {
                    original_image: String::new(), edited_image: String::new(),
                    orientation: args["orientation"].as_str().unwrap_or("horizontal").to_string(),
                    gap: args["gap"].as_u64().unwrap_or(20) as u32,
                    label_original: args.get("labels").and_then(|v| v.as_array()).and_then(|a| a.get(0)).and_then(|v| v.as_str()).map(|s| s.to_string()),
                    label_edited: args.get("labels").and_then(|v| v.as_array()).and_then(|a| a.get(1)).and_then(|v| v.as_str()).map(|s| s.to_string()),
                    label_size: 24,
                    label_color,
                    background_color: bg,
                };
                let comparison = create_comparison(&original, &edited, &params);
                let preview = generate_preview_result(&comparison).map_err(|e| e.to_string())?;
                (comparison, preview)
            }
            "pixel_diff" => {
                let original = Self::b64_to_img(args["originalImage"].as_str().ok_or("Missing originalImage")?).map_err(|e| e.to_string())?;
                let edited = Self::b64_to_img(args["editedImage"].as_str().ok_or("Missing editedImage")?).map_err(|e| e.to_string())?;
                let params = DiffParams { original_image: String::new(), edited_image: String::new(), threshold: args["threshold"].as_u64().unwrap_or(30) as u32 };
                let diff = create_diff(&original, &edited, &params);
                let preview = generate_preview_result(&diff).map_err(|e| e.to_string())?;
                (diff, preview)
            }
            _ => return Err(format!("Unknown diff mode: {}", mode)),
        };

        let b64 = Self::img_to_b64(&img).map_err(|e| e.to_string())?;

        Ok(CallToolResult::success(vec![
            RawContent::image(b64, "image/png").no_annotation(),
            RawContent::text(format!(
                "Comparison created. Full size: {}x{}. Preview: {}x{}",
                img.width(), img.height(), preview.width, preview.height
            )).no_annotation(),
            RawContent::image(preview.data, &preview.mime_type).no_annotation(),
        ]))
    }

    async fn handle_generate_image(&self, args: Value) -> Result<CallToolResult, anyhow::Error> {
        let params = GenerateImageParams {
            prompt: args["prompt"].as_str().unwrap_or("").to_string(),
            width: args["width"].as_u64().unwrap_or(1024) as u32,
            height: args["height"].as_u64().unwrap_or(1024) as u32,
            model: args["model"].as_str().unwrap_or("dall-e-3").to_string(),
            quality: args["quality"].as_str().unwrap_or("standard").to_string(),
            style: args["style"].as_str().unwrap_or("vivid").to_string(),
            api_key: None,
        };

        let img = generate_image(&params).await?;
        let b64 = Self::img_to_b64(&img)?;

        Ok(CallToolResult::success(vec![
            RawContent::image(b64, "image/png").no_annotation(),
            RawContent::text(format!(
                "Generated image from prompt: '{}'. Size: {}x{}",
                params.prompt, img.width(), img.height()
            )).no_annotation(),
        ]))
    }

    async fn handle_fill_with_generation(&self, args: Value) -> Result<CallToolResult, anyhow::Error> {
        let params = FillWithGenerationParams {
            prompt: args["prompt"].as_str().unwrap_or("").to_string(),
            original_image_b64: args["image"].as_str().unwrap_or("").to_string(),
            mask_image_b64: args["mask"].as_str().unwrap_or("").to_string(),
            model: args["model"].as_str().unwrap_or("gpt-image-1").to_string(),
            api_key: None,
        };

        let img = fill_with_generation(&params).await?;
        let b64 = Self::img_to_b64(&img)?;

        Ok(CallToolResult::success(vec![
            RawContent::image(b64, "image/png").no_annotation(),
            RawContent::text(format!(
                "Filled masked region with generated content for: '{}'. Size: {}x{}",
                params.prompt, img.width(), img.height()
            )).no_annotation(),
        ]))
    }

    async fn handle_edit_image_ai(&self, args: Value) -> Result<CallToolResult, anyhow::Error> {
        let params = crate::operations::generation::FalEditParams {
            prompt: args["prompt"].as_str().unwrap_or("").to_string(),
            image_b64: args["image"].as_str().unwrap_or("").to_string(),
            mask_b64: args.get("mask").and_then(|v| v.as_str()).map(|s| s.to_string()),
            quality: args["quality"].as_str().unwrap_or("high").to_string(),
            num_images: args["numImages"].as_u64().unwrap_or(1) as u32,
            output_format: args["outputFormat"].as_str().unwrap_or("png").to_string(),
            image_size: args["imageSize"].as_str().unwrap_or("auto").to_string(),
            api_key: None,
        };

        let img = crate::operations::generation::fal_edit_image(&params).await?;
        let b64 = Self::img_to_b64(&img)?;

        Ok(CallToolResult::success(vec![
            RawContent::image(b64, "image/png").no_annotation(),
            RawContent::text(format!(
                "Edited image via GPT Image 2. Size: {}x{}",
                img.width(), img.height()
            )).no_annotation(),
        ]))
    }

    async fn handle_segment_image(&self, args: Value) -> Result<CallToolResult, anyhow::Error> {
        let params = SegmentImageParams {
            image_b64: args["image"].as_str().unwrap_or("").to_string(),
            api_key: None,
            model: args["model"].as_str().unwrap_or("meta/sam-2:fe97b453a6455861e3bac769b441ca1f1086110da7466dbb65cf1eecfd60dc83").to_string(),
        };

        let (combined_b64, masks_b64) = segment_images_to_b64(&params).await?;
        let mut contents: Vec<Annotated<RawContent>> = vec![
            RawContent::image(combined_b64, "image/png").no_annotation(),
            RawContent::text(format!(
                "Segmented image. Found {} individual masks.",
                masks_b64.len()
            )).no_annotation(),
        ];
        for (i, mask_b64) in masks_b64.iter().enumerate() {
            contents.push(
                RawContent::image(mask_b64.clone(), "image/png").no_annotation()
            );
            contents.push(
                RawContent::text(format!("Mask {}", i + 1)).no_annotation()
            );
        }
        Ok(CallToolResult::success(contents))
    }

    async fn handle_remove_background(&self, args: Value) -> Result<CallToolResult, anyhow::Error> {
        let params = RemoveBackgroundParams {
            image_b64: args["image"].as_str().unwrap_or("").to_string(),
            api_key: None,
        };
        let b64 = remove_background_to_b64(&params).await?;
        Ok(CallToolResult::success(vec![
            RawContent::image(b64.clone(), "image/png").no_annotation(),
            RawContent::text("Background removed.").no_annotation(),
            RawContent::image(b64, "image/png").no_annotation(),
        ]))
    }

    fn handle_run_scheme(&self, args: Value) -> Result<CallToolResult, String> {
        let input_b64 = args.get("inputImage").and_then(|v| v.as_str());
        let scheme = args["script"].as_str().unwrap_or("");
        let (_, b64) = crate::steel_engine::run_scheme_script(scheme, input_b64)
            .map_err(|e| e)?;
        Ok(CallToolResult::success(vec![
            RawContent::image(b64, "image/png").no_annotation(),
        ]))
    }
}

pub async fn run_stdio_server() -> anyhow::Result<()> {
    tracing::info!("MCP Image Design server started (stdio via rmcp)");
    let server = ImageDesignServer::new();
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
