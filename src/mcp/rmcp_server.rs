use crate::operations::preview::*;
use crate::operations::*;
use axum::Router;
use image::{DynamicImage, GenericImageView, ImageFormat};
use rmcp::{
    ErrorData, ServerHandler, ServiceExt,
    model::*,
    service::RequestContext,
    transport::stdio,
    transport::streamable_http_server::{
        session::local::LocalSessionManager,
        tower::StreamableHttpService,
        StreamableHttpServerConfig,
    },
};
use serde_json::{json, Value};
use std::net::SocketAddr;
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
            "seamagic MCP server: view image diffs, or run Steel Scheme scripts for creative image editing.".to_string(),
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
            "view_diff" => self.handle_view_diff(args).await.map_err(|e|
                ErrorData::internal_error(format!("[view_diff] {}", e), None)),
            "run_steel" => self.handle_run_steel(args).map_err(|e|
                ErrorData::internal_error(format!("[run_steel] {}", e), None)),
            "edit_image" => self.handle_edit_image(args).map_err(|e|
                ErrorData::internal_error(format!("[edit_image] {}", e), None)),
            "create_image" => self.handle_create_image(args).map_err(|e|
                ErrorData::internal_error(format!("[create_image] {}", e), None)),
            "get_info" => self.handle_get_info(args).map_err(|e|
                ErrorData::internal_error(format!("[get_info] {}", e), None)),
            _ => Err(ErrorData::invalid_params(
                format!("Unknown tool '{}'. Available tools: view_diff, run_steel, edit_image, create_image, get_info", name),
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
                "view_diff",
                "Compare image(s). Modes: side_by_side, grid, pixel_diff. Returns a visual comparison + compressed preview.",
                Arc::new(json!({
                    "type": "object",
                    "properties": {
                        "mode": {
                            "type": "string",
                            "enum": ["side_by_side", "grid", "pixel_diff"],
                            "description": "side_by_side: two images next to each other. grid: step-by-step grid layout. pixel_diff: highlight pixel-level differences."
                        },
                        "originalImage": { "type": "string", "description": "Base64 original image (for side_by_side / pixel_diff)" },
                        "editedImage": { "type": "string", "description": "Base64 edited image (for side_by_side / pixel_diff)" },
                        "images": { "type": "array", "items": { "type": "string" }, "description": "Array of base64 images (for grid mode)" },
                        "labels": { "type": "array", "items": { "type": "string" }, "description": "Labels for each image in grid mode" },
                        "width": { "type": "integer", "description": "Grid canvas width (grid mode only)" },
                        "height": { "type": "integer", "description": "Grid canvas height (grid mode only)" },
                        "orientation": { "type": "string", "enum": ["horizontal","vertical"], "default": "horizontal", "description": "side_by_side orientation" },
                        "gap": { "type": "integer", "default": 20, "description": "Gap between images" },
                        "maxColumns": { "type": "integer", "default": 4, "description": "Max columns in grid" },
                        "threshold": { "type": "integer", "default": 30, "description": "Pixel diff threshold (0-255)" },
                        "backgroundColor": { "type": "string", "default": "#1a1a2e" },
                        "labelColor": { "type": "string", "default": "#FFFFFF" }
                    },
                    "required": ["mode"]
                }).as_object().unwrap().clone()),
            ),
            Tool::new(
                "run_steel",
                "Execute a Steel Scheme script for complex multi-step image creation/editing. The script has access to all image operations. If inputImage is provided, it is available as the string \"input\" in Scheme. The final image must be assigned to a variable named `result` or `output`.",
                Arc::new(json!({
                    "type": "object",
                    "properties": {
                        "script": {
                            "type": "string",
                            "description": "Steel Scheme code. Example: (define base \"input\") (define wavy (image-warp base \"wave\" 15.0 0.08)) (define result (image-posterize wavy 6))"
                        },
                        "inputImage": {
                            "type": "string",
                            "description": "Optional base64 image. Accessed in Scheme as the string \"input\". Example: (define base \"input\")"
                        }
                    },
                    "required": ["script"]
                }).as_object().unwrap().clone()),
            ),
            Tool::new(
                "edit_image",
                "Apply a single image operation (blur, crop, resize, tint, etc.) to a base64 image. Returns the edited image + a preview. For multi-step pipelines, use run_steel instead.",
                Arc::new(json!({
                    "type": "object",
                    "properties": {
                        "image": { "type": "string", "description": "Base64-encoded input image (PNG/JPG)" },
                        "operation": {
                            "type": "string",
                            "enum": ["blur", "brightness", "contrast", "grayscale", "tint", "vignette", "duotone", "crop", "crop_center", "resize", "rotate", "flip", "draw_text", "draw_rect", "draw_circle", "noise", "drop_shadow"],
                            "description": "Operation to apply. Only relevant params for the chosen operation are used."
                        },
                        "sigma": { "type": "number", "default": 1.0, "description": "Blur sigma (blur)" },
                        "value": { "type": "number", "default": 0.0, "description": "Brightness/contrast value (brightness, contrast)" },
                        "color": { "type": "string", "default": "#FF0000", "description": "Tint or shape color (tint, draw_text, draw_rect, draw_circle)" },
                        "amount": { "type": "number", "default": 0.5, "description": "Tint amount 0-1 (tint)" },
                        "strength": { "type": "number", "default": 0.5, "description": "Vignette strength 0-1 (vignette)" },
                        "radius": { "type": "number", "default": 1.0, "description": "Vignette radius / circle radius (vignette, draw_circle)" },
                        "vignetteColor": { "type": "string", "description": "Optional vignette color (vignette)" },
                        "shadowColor": { "type": "string", "default": "#000000", "description": "Shadow color (drop_shadow)" },
                        "highlightColor": { "type": "string", "default": "#FFFFFF", "description": "Highlight color (duotone)" },
                        "x": { "type": "integer", "default": 0, "description": "X position (crop, draw_text, draw_rect, draw_circle)" },
                        "y": { "type": "integer", "default": 0, "description": "Y position (crop, draw_text, draw_rect, draw_circle)" },
                        "width": { "type": "integer", "description": "Width (crop, crop_center, resize, draw_rect)" },
                        "height": { "type": "integer", "description": "Height (crop, crop_center, resize, draw_rect)" },
                        "resizeMode": { "type": "string", "enum": ["exact", "fit", "fill", "scale"], "default": "exact", "description": "Resize mode (resize)" },
                        "degrees": { "type": "number", "default": 0.0, "description": "Rotation degrees (rotate)" },
                        "bgColor": { "type": "string", "description": "Background color for rotation (rotate)" },
                        "horizontal": { "type": "boolean", "default": false, "description": "Flip horizontally (flip)" },
                        "vertical": { "type": "boolean", "default": false, "description": "Flip vertically (flip)" },
                        "text": { "type": "string", "default": "", "description": "Text to draw (draw_text)" },
                        "size": { "type": "integer", "default": 24, "description": "Font size (draw_text) or shape size" },
                        "filled": { "type": "boolean", "default": true, "description": "Fill shape (draw_rect, draw_circle)" },
                        "noiseType": { "type": "string", "default": "grain", "description": "Noise type (noise)" },
                        "seed": { "type": "integer", "default": 42, "description": "Noise seed (noise)" },
                        "offsetX": { "type": "integer", "default": 5, "description": "Shadow offset X (drop_shadow)" },
                        "offsetY": { "type": "integer", "default": 5, "description": "Shadow offset Y (drop_shadow)" },
                        "blurRadius": { "type": "integer", "default": 5, "description": "Shadow blur radius (drop_shadow)" }
                    },
                    "required": ["image", "operation"]
                }).as_object().unwrap().clone()),
            ),
            Tool::new(
                "create_image",
                "Generate a new image from scratch: solid canvas, gradient, or framed canvas. No input image needed.",
                Arc::new(json!({
                    "type": "object",
                    "properties": {
                        "type": {
                            "type": "string",
                            "enum": ["canvas", "gradient", "frame"],
                            "description": "canvas: solid color, gradient: linear gradient, frame: bordered canvas"
                        },
                        "width": { "type": "integer", "default": 512, "description": "Image width" },
                        "height": { "type": "integer", "default": 512, "description": "Image height" },
                        "color": { "type": "string", "default": "#000000", "description": "Canvas color (canvas, frame border)" },
                        "startColor": { "type": "string", "default": "#000000", "description": "Gradient start color (gradient)" },
                        "endColor": { "type": "string", "default": "#FFFFFF", "description": "Gradient end color (gradient)" },
                        "angle": { "type": "number", "default": 0.0, "description": "Gradient angle in degrees (gradient)" },
                        "frameWidth": { "type": "integer", "default": 20, "description": "Frame border width (frame)" }
                    },
                    "required": ["type"]
                }).as_object().unwrap().clone()),
            ),
            Tool::new(
                "get_info",
                "Get dimensions and basic metadata of a base64 image.",
                Arc::new(json!({
                    "type": "object",
                    "properties": {
                        "image": { "type": "string", "description": "Base64-encoded image" }
                    },
                    "required": ["image"]
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
    async fn handle_view_diff(&self, args: Value) -> Result<CallToolResult, String> {
        let mode = args["mode"].as_str().unwrap_or("side_by_side");
        let bg = args["backgroundColor"].as_str().unwrap_or("#1a1a2e").to_string();
        let label_color = args["labelColor"].as_str().unwrap_or("#FFFFFF").to_string();

        let (img, preview, text) = match mode {
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
                (comparison, preview, "Side-by-side comparison created".to_string())
            }
            "pixel_diff" => {
                let original = Self::b64_to_img(args["originalImage"].as_str().ok_or("Missing originalImage")?).map_err(|e| e.to_string())?;
                let edited = Self::b64_to_img(args["editedImage"].as_str().ok_or("Missing editedImage")?).map_err(|e| e.to_string())?;
                let params = DiffParams {
                    original_image: String::new(), edited_image: String::new(),
                    threshold: args["threshold"].as_u64().unwrap_or(30) as u32,
                };
                let diff = create_diff(&original, &edited, &params);
                let preview = generate_preview_result(&diff).map_err(|e| e.to_string())?;
                (diff, preview, "Pixel diff created — cyan = changed, dim = unchanged".to_string())
            }
            "grid" => {
                let images_b64: Vec<String> = serde_json::from_value(args["images"].clone()).map_err(|e| format!("Invalid images: {}", e))?;
                if images_b64.is_empty() {
                    return Err("grid mode requires at least one image".to_string());
                }
                let mut images = Vec::new();
                for b64 in &images_b64 { images.push(load_from_base64(b64).map_err(|e| e.to_string())?); }
                let labels: Option<Vec<String>> = args.get("labels")
                    .and_then(|v| serde_json::from_value(v.clone()).ok());
                let params = StepsGridParams {
                    images: Vec::new(),
                    labels,
                    width: args["width"].as_u64().unwrap_or(2000) as u32,
                    height: args["height"].as_u64().unwrap_or(600) as u32,
                    gap: args["gap"].as_u64().unwrap_or(10) as u32,
                    max_columns: args["maxColumns"].as_u64().unwrap_or(4) as u32,
                    label_size: 24,
                    label_color,
                    background_color: bg,
                };
                let grid = create_steps_grid(&images, &params);
                let preview = generate_preview_result(&grid).map_err(|e| e.to_string())?;
                (grid, preview, format!("Grid comparison created with {} images", images_b64.len()))
            }
            _ => return Err(format!("Unknown diff mode: {}", mode)),
        };

        let b64 = Self::img_to_b64(&img).map_err(|e| e.to_string())?;

        Ok(CallToolResult::success(vec![
            RawContent::text(text).no_annotation(),
            RawContent::image(b64, "image/png").no_annotation(),
            RawContent::text(format!("Full: {}x{} | Preview: {}x{}",
                img.width(), img.height(), preview.width, preview.height)).no_annotation(),
            RawContent::image(preview.data, &preview.mime_type).no_annotation(),
        ]))
    }

    fn handle_run_steel(&self, args: Value) -> Result<CallToolResult, String> {
        let input_b64 = args.get("inputImage").and_then(|v| v.as_str());
        let scheme = args["script"].as_str().ok_or("Missing 'script' field")?;
        let (_, b64) = crate::steel_engine::run_scheme_script(scheme, input_b64)
            .map_err(|e| e)?;
        Ok(CallToolResult::success(vec![
            RawContent::image(b64, "image/png").no_annotation(),
        ]))
    }

    fn handle_edit_image(&self, args: Value) -> Result<CallToolResult, String> {
        let img = Self::b64_to_img(args["image"].as_str().ok_or("Missing 'image' field")?)
            .map_err(|e| e.to_string())?;
        let op = args["operation"].as_str().ok_or("Missing 'operation' field")?;

        let result = match op {
            "blur" => {
                let sigma = args["sigma"].as_f64().unwrap_or(1.0) as f32;
                blur(&img, &BlurParams { sigma })
            }
            "brightness" => {
                let value = args["value"].as_f64().unwrap_or(0.0) as f32;
                brightness(&img, &BrightnessParams { value })
            }
            "contrast" => {
                let value = args["value"].as_f64().unwrap_or(0.0) as f32;
                filters::contrast(&img, &ContrastParams { value })
            }
            "grayscale" => grayscale(&img, &GrayscaleParams { mode: "luma".to_string() }),
            "tint" => {
                let color = args["color"].as_str().unwrap_or("#FF0000").to_string();
                let amount = args["amount"].as_f64().unwrap_or(0.5) as f32;
                tint(&img, &TintParams { color, amount })
            }
            "vignette" => {
                let strength = args["strength"].as_f64().unwrap_or(0.5) as f32;
                let radius = args["radius"].as_f64().unwrap_or(1.0) as f32;
                let color = args.get("vignetteColor").and_then(|v| v.as_str()).map(|s| s.to_string());
                vignette(&img, &VignetteParams { strength, radius, color })
            }
            "duotone" => {
                let shadow = args["shadowColor"].as_str().unwrap_or("#000000").to_string();
                let highlight = args["highlightColor"].as_str().unwrap_or("#FFFFFF").to_string();
                duotone(&img, &DuotoneParams { shadow_color: shadow, highlight_color: highlight })
            }
            "crop" => {
                let x = args["x"].as_u64().unwrap_or(0) as u32;
                let y = args["y"].as_u64().unwrap_or(0) as u32;
                let w = args["width"].as_u64().ok_or("Missing 'width'")? as u32;
                let h = args["height"].as_u64().ok_or("Missing 'height'")? as u32;
                crop(&img, &CropParams { x, y, width: w, height: h })
            }
            "crop_center" => {
                let w = args["width"].as_u64().ok_or("Missing 'width'")? as u32;
                let h = args["height"].as_u64().ok_or("Missing 'height'")? as u32;
                crop_from_center(&img, &CropFromCenterParams { width: w, height: h })
            }
            "resize" => {
                let w = args["width"].as_u64().map(|v| v as u32);
                let h = args["height"].as_u64().map(|v| v as u32);
                let mode_str = args["resizeMode"].as_str().unwrap_or("exact");
                let mode = match mode_str {
                    "fit" => ResizeMode::Fit,
                    "fill" => ResizeMode::Fill,
                    "scale" => ResizeMode::Scale,
                    _ => ResizeMode::Exact,
                };
                resize(&img, &ResizeParams { width: w, height: h, mode, filter: "lanczos3".to_string() })
            }
            "rotate" => {
                let degrees = args["degrees"].as_f64().unwrap_or(0.0) as f32;
                let bg = args.get("bgColor").and_then(|v| v.as_str()).map(|s| s.to_string());
                rotate(&img, &RotateParams { degrees, background_color: bg })
            }
            "flip" => {
                let h = args["horizontal"].as_bool().unwrap_or(false);
                let v = args["vertical"].as_bool().unwrap_or(false);
                flip(&img, &FlipParams { horizontal: h, vertical: v })
            }
            "draw_text" => {
                let text = args["text"].as_str().unwrap_or("").to_string();
                let x = args["x"].as_i64().unwrap_or(0) as i32;
                let y = args["y"].as_i64().unwrap_or(0) as i32;
                let color = args["color"].as_str().unwrap_or("#000000").to_string();
                let font_size = args["size"].as_u64().unwrap_or(24) as u32;
                draw_text(&img, &TextParams { text, x, y, color, font_size, align: "left".to_string(), background_color: None, max_width: None })
            }
            "draw_rect" => {
                let x = args["x"].as_i64().unwrap_or(0) as i32;
                let y = args["y"].as_i64().unwrap_or(0) as i32;
                let w = args["width"].as_u64().ok_or("Missing 'width'")? as u32;
                let h = args["height"].as_u64().ok_or("Missing 'height'")? as u32;
                let color = args["color"].as_str().unwrap_or("#000000").to_string();
                let filled = args["filled"].as_bool().unwrap_or(true);
                draw_rectangle(&img, &RectangleParams { x, y, width: w, height: h, color, filled, border_radius: 0, stroke_width: 0, stroke_color: None })
            }
            "draw_circle" => {
                let x = args["x"].as_i64().unwrap_or(0) as i32;
                let y = args["y"].as_i64().unwrap_or(0) as i32;
                let radius = args["radius"].as_u64().unwrap_or(10) as u32;
                let color = args["color"].as_str().unwrap_or("#000000").to_string();
                let filled = args["filled"].as_bool().unwrap_or(true);
                draw_circle(&img, &CircleMaskParams { x, y, radius, color, filled, stroke_width: 0, stroke_color: None })
            }
            "noise" => {
                let amount = args["value"].as_f64().unwrap_or(0.1) as f32;
                let noise_type = args["noiseType"].as_str().unwrap_or("grain").to_string();
                let seed = args["seed"].as_u64().unwrap_or(42) as u32;
                noise(&img, &NoiseParams { amount, noise_type, seed })
            }
            "drop_shadow" => {
                let offset_x = args["offsetX"].as_i64().unwrap_or(5) as i32;
                let offset_y = args["offsetY"].as_i64().unwrap_or(5) as i32;
                let blur_radius = args["blurRadius"].as_u64().unwrap_or(5) as u32;
                let color = args["color"].as_str().unwrap_or("#000000").to_string();
                drop_shadow(&img, &ShadowParams { offset_x, offset_y, blur_radius, color })
            }
            _ => return Err(format!("Unknown edit operation: {}", op)),
        };

        let b64 = Self::img_to_b64(&result).map_err(|e| e.to_string())?;
        let preview = generate_preview_result(&result).map_err(|e| e.to_string())?;
        Ok(CallToolResult::success(vec![
            RawContent::text(format!("Applied {} — {}x{}", op, result.width(), result.height())).no_annotation(),
            RawContent::image(b64, "image/png").no_annotation(),
            RawContent::image(preview.data, &preview.mime_type).no_annotation(),
        ]))
    }

    fn handle_create_image(&self, args: Value) -> Result<CallToolResult, String> {
        let kind = args["type"].as_str().ok_or("Missing 'type' field")?;
        let width = args["width"].as_u64().unwrap_or(512) as u32;
        let height = args["height"].as_u64().unwrap_or(512) as u32;

        let result = match kind {
            "canvas" => {
                let color = args["color"].as_str().unwrap_or("#000000").to_string();
                let rgba = super::super::operations::utils::hex_to_rgba(&color).map_err(|e| e.to_string())?;
                let mut canvas = image::RgbaImage::new(width, height);
                for px in canvas.pixels_mut() { *px = rgba; }
                DynamicImage::ImageRgba8(canvas)
            }
            "gradient" => {
                let start = args["startColor"].as_str().unwrap_or("#000000").to_string();
                let end = args["endColor"].as_str().unwrap_or("#FFFFFF").to_string();
                let angle = args["angle"].as_f64().unwrap_or(0.0) as f32;
                super::super::operations::create_gradient(&GradientParams { width, height, start_color: start, end_color: end, angle })
            }
            "frame" => {
                let color = args["color"].as_str().unwrap_or("#000000").to_string();
                let frame_width = args["frameWidth"].as_u64().unwrap_or(20) as u32;
                super::super::operations::create_frame(&FrameParams { width, height, frame_width, frame_color: color, inner_color: None })
            }
            _ => return Err(format!("Unknown create type: {}", kind)),
        };

        let b64 = Self::img_to_b64(&result).map_err(|e| e.to_string())?;
        let preview = generate_preview_result(&result).map_err(|e| e.to_string())?;
        Ok(CallToolResult::success(vec![
            RawContent::text(format!("Created {} — {}x{}", kind, width, height)).no_annotation(),
            RawContent::image(b64, "image/png").no_annotation(),
            RawContent::image(preview.data, &preview.mime_type).no_annotation(),
        ]))
    }

    fn handle_get_info(&self, args: Value) -> Result<CallToolResult, String> {
        let img = Self::b64_to_img(args["image"].as_str().ok_or("Missing 'image' field")?)
            .map_err(|e| e.to_string())?;
        let (w, h) = img.dimensions();
        Ok(CallToolResult::success(vec![
            RawContent::text(format!("Dimensions: {}x{} pixels\nFormat: PNG (output)", w, h)).no_annotation(),
        ]))
    }
}

pub async fn run_stdio_server() -> anyhow::Result<()> {
    tracing::info!("seamagic MCP server started (stdio via rmcp)");
    let server = ImageDesignServer::new();
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

pub async fn run_http_server(bind: SocketAddr, path: &str) -> anyhow::Result<()> {
    tracing::info!("seamagic MCP server starting on http://{bind}{path}");
    let server = ImageDesignServer::new();
    let mut config = StreamableHttpServerConfig::default();
    config.allowed_hosts = allowed_hosts_for(bind);

    let normalized_path = normalize_path(path);
    let factory = move || Ok(server.clone());
    let service = StreamableHttpService::new(factory, Arc::new(LocalSessionManager::default()), config);
    let app = Router::new().route_service(&normalized_path, service);
    let listener = tokio::net::TcpListener::bind(bind).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn normalize_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() || trimmed == "/" {
        return "/mcp".to_string();
    }
    if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{}", trimmed)
    }
}

fn allowed_hosts_for(bind_addr: SocketAddr) -> Vec<String> {
    let host = bind_addr.ip().to_string();
    let port = bind_addr.port();
    vec![
        "localhost".to_string(),
        format!("localhost:{port}"),
        "127.0.0.1".to_string(),
        format!("127.0.0.1:{port}"),
        "::1".to_string(),
        format!("[::1]:{port}"),
        host.clone(),
        format!("{host}:{port}"),
    ]
}
