use clap::{Parser, Subcommand};
use image::{DynamicImage, GenericImageView, ImageFormat, Rgba, RgbaImage};
use std::net::SocketAddr;
use std::path::PathBuf;

use seamagic::operations::*;

#[derive(Parser)]
#[command(name = "seamagic")]
#[command(about = "Seamagic - A CLI for image design and manipulation")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Start MCP stdio server")]
    Mcp,

    #[command(about = "Start MCP HTTP server")]
    HttpMcp {
        /// Address to bind (default: 0.0.0.0:3000)
        #[arg(long, default_value = "0.0.0.0:3000")]
        bind: SocketAddr,

        /// Route path (default: /mcp)
        #[arg(long, default_value = "/mcp")]
        path: String,
    },

    #[command(about = "Get image dimensions")]
    Info {
        input: PathBuf,
    },

    #[command(about = "Crop an image")]
    Crop {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long)]
        x: u32,
        #[arg(long)]
        y: u32,
        #[arg(long)]
        width: u32,
        #[arg(long)]
        height: u32,
    },

    #[command(about = "Crop from center")]
    CropCenter {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long)]
        width: u32,
        #[arg(long)]
        height: u32,
    },

    #[command(about = "Smart crop with optional aspect ratio")]
    SmartCrop {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long)]
        ratio: Option<f32>,
        #[arg(long)]
        width: Option<u32>,
        #[arg(long)]
        height: Option<u32>,
    },

    #[command(about = "Resize an image")]
    Resize {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long)]
        width: Option<u32>,
        #[arg(long)]
        height: Option<u32>,
        #[arg(long, value_enum, default_value = "exact")]
        mode: ResizeModeCli,
        #[arg(long, default_value = "lanczos3")]
        filter: String,
    },

    #[command(about = "Create a thumbnail")]
    Thumbnail {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long)]
        size: u32,
    },

    #[command(about = "Rotate an image")]
    Rotate {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long)]
        degrees: f32,
        #[arg(long)]
        bg: Option<String>,
    },

    #[command(about = "Flip an image")]
    Flip {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long)]
        horizontal: bool,
        #[arg(long)]
        vertical: bool,
    },

    #[command(about = "Apply blur")]
    Blur {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long)]
        sigma: f32,
    },

    #[command(about = "Adjust brightness")]
    Brightness {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long)]
        value: f32,
    },

    #[command(about = "Adjust contrast")]
    Contrast {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long)]
        value: f32,
    },

    #[command(about = "Convert to grayscale")]
    Grayscale {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },

    #[command(about = "Tint with a color")]
    Tint {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long)]
        color: String,
        #[arg(long, default_value = "0.5")]
        amount: f32,
    },

    #[command(about = "Apply vignette")]
    Vignette {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long, default_value = "0.5")]
        strength: f32,
        #[arg(long, default_value = "1.0")]
        radius: f32,
        #[arg(long)]
        color: Option<String>,
    },

    #[command(about = "Apply duotone effect")]
    Duotone {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long, default_value = "#000000")]
        shadow: String,
        #[arg(long, default_value = "#ffffff")]
        highlight: String,
    },

    #[command(about = "Add noise")]
    Noise {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long, default_value = "0.1")]
        amount: f32,
        #[arg(long, default_value = "grain")]
        noise_type: String,
        #[arg(long, default_value = "42")]
        seed: u32,
    },

    #[command(about = "Apply liquify distortion")]
    Liquify {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long, default_value = "bulge")]
        mode: String,
        #[arg(long, default_value = "0.5")]
        center_x: f32,
        #[arg(long, default_value = "0.5")]
        center_y: f32,
        #[arg(long, default_value = "0.3")]
        radius: f32,
        #[arg(long, default_value = "0.5")]
        strength: f32,
    },

    #[command(about = "Apply warp distortion")]
    Warp {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long, default_value = "wave")]
        mode: String,
        #[arg(long, default_value = "10.0")]
        amplitude: f32,
        #[arg(long, default_value = "0.05")]
        frequency: f32,
    },

    #[command(about = "Apply glitch effect")]
    Glitch {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long, default_value = "rgb_split")]
        mode: String,
        #[arg(long, default_value = "0.5")]
        intensity: f32,
        #[arg(long, default_value = "42")]
        seed: u32,
    },

    #[command(about = "Apply scanlines")]
    Scanlines {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long, default_value = "4")]
        spacing: u32,
        #[arg(long, default_value = "0.3")]
        opacity: f32,
        #[arg(long, default_value = "1")]
        thickness: u32,
    },

    #[command(about = "Apply halftone")]
    Halftone {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long, default_value = "8")]
        dot_size: u32,
        #[arg(long, default_value = "45.0")]
        angle: f32,
        #[arg(long, default_value = "color")]
        mode: String,
    },

    #[command(about = "Apply posterize")]
    Posterize {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long, default_value = "6")]
        levels: u8,
    },

    #[command(about = "Apply edge detection")]
    EdgeDetect {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long, default_value = "20")]
        threshold: u8,
        #[arg(long)]
        invert: bool,
    },

    #[command(about = "Apply emboss")]
    Emboss {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long, default_value = "1.0")]
        strength: f32,
    },

    #[command(about = "Apply solarize")]
    Solarize {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long, default_value = "128")]
        threshold: u8,
    },

    #[command(about = "Apply chromatic aberration")]
    Chromatic {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long, default_value = "5.0")]
        shift: f32,
        #[arg(long)]
        auto: bool,
    },

    #[command(about = "Apply pixel sort")]
    PixelSort {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long, default_value = "horizontal")]
        axis: String,
        #[arg(long, default_value = "0.1")]
        threshold_low: f32,
        #[arg(long, default_value = "0.9")]
        threshold_high: f32,
    },

    #[command(about = "Apply kaleidoscope")]
    Kaleidoscope {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long, default_value = "6")]
        segments: u32,
        #[arg(long, default_value = "0.0")]
        offset: f32,
    },

    #[command(about = "Overlay one image on another")]
    Overlay {
        base: PathBuf,
        top: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long, default_value = "0")]
        x: i32,
        #[arg(long, default_value = "0")]
        y: i32,
        #[arg(long, default_value = "1.0")]
        opacity: f32,
        #[arg(long, default_value = "normal")]
        blend_mode: String,
    },

    #[command(about = "Draw text on an image")]
    DrawText {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long)]
        text: String,
        #[arg(long, default_value = "0")]
        x: i32,
        #[arg(long, default_value = "0")]
        y: i32,
        #[arg(long, default_value = "#000000")]
        color: String,
        #[arg(long, default_value = "24")]
        size: u32,
    },

    #[command(about = "Draw rectangle")]
    DrawRect {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long, default_value = "0")]
        x: i32,
        #[arg(long, default_value = "0")]
        y: i32,
        #[arg(long)]
        width: u32,
        #[arg(long)]
        height: u32,
        #[arg(long, default_value = "#000000")]
        color: String,
        #[arg(long)]
        filled: bool,
    },

    #[command(about = "Draw circle")]
    DrawCircle {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long, default_value = "0")]
        x: i32,
        #[arg(long, default_value = "0")]
        y: i32,
        #[arg(long)]
        radius: u32,
        #[arg(long, default_value = "#000000")]
        color: String,
        #[arg(long)]
        filled: bool,
    },

    #[command(about = "Create a color canvas")]
    Canvas {
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long)]
        width: u32,
        #[arg(long)]
        height: u32,
        #[arg(long, default_value = "#000000")]
        color: String,
    },

    #[command(about = "Create a gradient")]
    Gradient {
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long)]
        width: u32,
        #[arg(long)]
        height: u32,
        #[arg(long, default_value = "#000000")]
        start: String,
        #[arg(long, default_value = "#ffffff")]
        end: String,
        #[arg(long, default_value = "0.0")]
        angle: f32,
    },

    #[command(about = "Apply shape mask")]
    Mask {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long, default_value = "circle")]
        shape: String,
        #[arg(long, default_value = "0")]
        x: i32,
        #[arg(long, default_value = "0")]
        y: i32,
        #[arg(long)]
        width: u32,
        #[arg(long)]
        height: u32,
    },

    #[command(about = "Apply a pipeline of operations from a JSON config file")]
    Pipeline {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long)]
        config: PathBuf,
    },
}

#[derive(Clone, Debug, clap::ValueEnum)]
enum ResizeModeCli {
    Exact,
    Fit,
    Fill,
    Scale,
}

impl ResizeModeCli {
    fn to_resize_mode(&self) -> seamagic::operations::ResizeMode {
        match self {
            ResizeModeCli::Exact => seamagic::operations::ResizeMode::Exact,
            ResizeModeCli::Fit => seamagic::operations::ResizeMode::Fit,
            ResizeModeCli::Fill => seamagic::operations::ResizeMode::Fill,
            ResizeModeCli::Scale => seamagic::operations::ResizeMode::Scale,
        }
    }
}

fn load_image(path: &PathBuf) -> anyhow::Result<DynamicImage> {
    let img = image::open(path)?;
    Ok(img)
}

fn save_image(img: &DynamicImage, path: &PathBuf) -> anyhow::Result<()> {
    let format = match path.extension().and_then(|e| e.to_str()) {
        Some("png") => ImageFormat::Png,
        Some("jpg") | Some("jpeg") => ImageFormat::Jpeg,
        Some("webp") => ImageFormat::WebP,
        Some("gif") => ImageFormat::Gif,
        Some("bmp") => ImageFormat::Bmp,
        Some("tiff") => ImageFormat::Tiff,
        _ => ImageFormat::Png,
    };
    img.save_with_format(path, format)?;
    Ok(())
}

fn run_mcp_server() -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .with_ansi(false)
            .init();
        seamagic::mcp::run_stdio_server().await
    })
}

fn run_http_mcp_server(bind: SocketAddr, path: &str) -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .with_ansi(false)
            .init();
        seamagic::mcp::run_http_server(bind, path).await
    })
}

fn main() -> anyhow::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();

    // Default to MCP server for backward compatibility when invoked without args
    if args.len() == 1 {
        return run_mcp_server();
    }

    let cli = Cli::parse();

    match cli.command {
        Commands::Mcp => run_mcp_server(),

        Commands::HttpMcp { bind, path } => run_http_mcp_server(bind, &path),

        Commands::Info { input } => {
            let img = load_image(&input)?;
            let (w, h) = img.dimensions();
            println!("{}: {}x{}", input.display(), w, h);
            Ok(())
        }

        Commands::Crop { input, output, x, y, width, height } => {
            let img = load_image(&input)?;
            let params = CropParams { x, y, width, height };
            let result = crop(&img, &params);
            save_image(&result, &output)?;
            println!("Cropped to {}x{} at ({},{})", width, height, x, y);
            Ok(())
        }

        Commands::CropCenter { input, output, width, height } => {
            let img = load_image(&input)?;
            let params = CropFromCenterParams { width, height };
            let result = crop_from_center(&img, &params);
            save_image(&result, &output)?;
            println!("Center-cropped to {}x{}", width, height);
            Ok(())
        }

        Commands::SmartCrop { input, output, ratio, width, height } => {
            let img = load_image(&input)?;
            let params = SmartCropParams {
                aspect_ratio: ratio,
                target_width: width,
                target_height: height,
                focus_x: Some(0.5),
                focus_y: Some(0.5),
            };
            let result = smart_crop(&img, &params);
            save_image(&result, &output)?;
            println!("Smart-cropped to {}x{}", result.width(), result.height());
            Ok(())
        }

        Commands::Resize { input, output, width, height, mode, filter } => {
            let img = load_image(&input)?;
            let params = ResizeParams {
                width,
                height,
                mode: mode.to_resize_mode(),
                filter,
            };
            let result = resize(&img, &params);
            save_image(&result, &output)?;
            println!("Resized to {}x{}", result.width(), result.height());
            Ok(())
        }

        Commands::Thumbnail { input, output, size } => {
            let img = load_image(&input)?;
            let params = ThumbnailParams { size };
            let result = thumbnail(&img, &params);
            save_image(&result, &output)?;
            println!("Thumbnail {}x{}", result.width(), result.height());
            Ok(())
        }

        Commands::Rotate { input, output, degrees, bg } => {
            let img = load_image(&input)?;
            let params = RotateParams {
                degrees,
                background_color: bg,
            };
            let result = rotate(&img, &params);
            save_image(&result, &output)?;
            println!("Rotated {} degrees", degrees);
            Ok(())
        }

        Commands::Flip { input, output, horizontal, vertical } => {
            let img = load_image(&input)?;
            let params = FlipParams { horizontal, vertical };
            let result = flip(&img, &params);
            save_image(&result, &output)?;
            println!("Flipped h:{} v:{}", horizontal, vertical);
            Ok(())
        }

        Commands::Blur { input, output, sigma } => {
            let img = load_image(&input)?;
            let params = BlurParams { sigma };
            let result = blur(&img, &params);
            save_image(&result, &output)?;
            println!("Blurred with sigma {}", sigma);
            Ok(())
        }

        Commands::Brightness { input, output, value } => {
            let img = load_image(&input)?;
            let params = BrightnessParams { value };
            let result = brightness(&img, &params);
            save_image(&result, &output)?;
            println!("Brightness adjusted by {}", value);
            Ok(())
        }

        Commands::Contrast { input, output, value } => {
            let img = load_image(&input)?;
            let params = ContrastParams { value };
            let result = contrast(&img, &params);
            save_image(&result, &output)?;
            println!("Contrast adjusted by {}", value);
            Ok(())
        }

        Commands::Grayscale { input, output } => {
            let img = load_image(&input)?;
            let params = GrayscaleParams { mode: "luma".to_string() };
            let result = grayscale(&img, &params);
            save_image(&result, &output)?;
            println!("Converted to grayscale");
            Ok(())
        }

        Commands::Tint { input, output, color, amount } => {
            let img = load_image(&input)?;
            let params = TintParams { color, amount };
            let result = tint(&img, &params);
            save_image(&result, &output)?;
            println!("Tinted with {}", params.color);
            Ok(())
        }

        Commands::Vignette { input, output, strength, radius, color } => {
            let img = load_image(&input)?;
            let params = VignetteParams { strength, radius, color };
            let result = vignette(&img, &params);
            save_image(&result, &output)?;
            println!("Applied vignette");
            Ok(())
        }

        Commands::Duotone { input, output, shadow, highlight } => {
            let img = load_image(&input)?;
            let params = DuotoneParams {
                shadow_color: shadow,
                highlight_color: highlight,
            };
            let result = duotone(&img, &params);
            save_image(&result, &output)?;
            println!("Applied duotone");
            Ok(())
        }

        Commands::Noise { input, output, amount, noise_type, seed } => {
            let img = load_image(&input)?;
            let params = NoiseParams { amount, noise_type: noise_type.clone(), seed };
            let result = noise(&img, &params);
            save_image(&result, &output)?;
            println!("Added {} noise", noise_type);
            Ok(())
        }

        Commands::Liquify { input, output, mode, center_x, center_y, radius, strength } => {
            let img = load_image(&input)?;
            let params = LiquifyParams { mode, center_x, center_y, radius, strength };
            let result = liquify(&img, &params);
            save_image(&result, &output)?;
            println!("Applied liquify {}", params.mode);
            Ok(())
        }

        Commands::Warp { input, output, mode, amplitude, frequency } => {
            let img = load_image(&input)?;
            let params = WarpParams { mode, amplitude, frequency };
            let result = warp(&img, &params);
            save_image(&result, &output)?;
            println!("Applied warp {}", params.mode);
            Ok(())
        }

        Commands::Glitch { input, output, mode, intensity, seed } => {
            let img = load_image(&input)?;
            let params = GlitchParams { mode, intensity, seed };
            let result = glitch(&img, &params);
            save_image(&result, &output)?;
            println!("Applied glitch {}", params.mode);
            Ok(())
        }

        Commands::Scanlines { input, output, spacing, opacity, thickness } => {
            let img = load_image(&input)?;
            let params = ScanlinesParams { spacing, opacity, thickness };
            let result = scanlines(&img, &params);
            save_image(&result, &output)?;
            println!("Applied scanlines");
            Ok(())
        }

        Commands::Halftone { input, output, dot_size, angle, mode } => {
            let img = load_image(&input)?;
            let params = HalftoneParams { dot_size, angle, mode };
            let result = halftone(&img, &params);
            save_image(&result, &output)?;
            println!("Applied halftone");
            Ok(())
        }

        Commands::Posterize { input, output, levels } => {
            let img = load_image(&input)?;
            let params = PosterizeParams { levels };
            let result = posterize(&img, &params);
            save_image(&result, &output)?;
            println!("Posterized to {} levels", levels);
            Ok(())
        }

        Commands::EdgeDetect { input, output, threshold, invert } => {
            let img = load_image(&input)?;
            let params = EdgeDetectParams { threshold, invert };
            let result = edge_detect(&img, &params);
            save_image(&result, &output)?;
            println!("Applied edge detection");
            Ok(())
        }

        Commands::Emboss { input, output, strength } => {
            let img = load_image(&input)?;
            let params = EmbossParams { strength };
            let result = emboss(&img, &params);
            save_image(&result, &output)?;
            println!("Applied emboss");
            Ok(())
        }

        Commands::Solarize { input, output, threshold } => {
            let img = load_image(&input)?;
            let params = SolarizeParams { threshold };
            let result = solarize(&img, &params);
            save_image(&result, &output)?;
            println!("Applied solarize");
            Ok(())
        }

        Commands::Chromatic { input, output, shift, auto } => {
            let img = load_image(&input)?;
            let params = ChromaticAberrationParams { shift, auto };
            let result = chromatic_aberration(&img, &params);
            save_image(&result, &output)?;
            println!("Applied chromatic aberration");
            Ok(())
        }

        Commands::PixelSort { input, output, axis, threshold_low, threshold_high } => {
            let img = load_image(&input)?;
            let params = PixelSortParams { axis, threshold_low, threshold_high };
            let result = pixel_sort(&img, &params);
            save_image(&result, &output)?;
            println!("Applied pixel sort");
            Ok(())
        }

        Commands::Kaleidoscope { input, output, segments, offset } => {
            let img = load_image(&input)?;
            let params = KaleidoscopeParams { segments, offset };
            let result = kaleidoscope(&img, &params);
            save_image(&result, &output)?;
            println!("Applied kaleidoscope");
            Ok(())
        }

        Commands::Overlay { base, top, output, x, y, opacity, blend_mode } => {
            let base_img = load_image(&base)?;
            let top_img = load_image(&top)?;
            let params = OverlayParams {
                base_image: String::new(),
                overlay_image: String::new(),
                x,
                y,
                opacity: Some(opacity),
                blend_mode: Some(blend_mode),
            };
            let result = overlay(&base_img, &top_img, &params);
            save_image(&result, &output)?;
            println!("Overlayed {} on {}", top.display(), base.display());
            Ok(())
        }

        Commands::DrawText { input, output, text, x, y, color, size } => {
            let img = load_image(&input)?;
            let params = TextParams {
                text,
                x,
                y,
                color,
                font_size: size,
                background_color: None,
                align: "left".to_string(),
                max_width: None,
            };
            let result = draw_text(&img, &params);
            save_image(&result, &output)?;
            println!("Drew text on image");
            Ok(())
        }

        Commands::DrawRect { input, output, x, y, width, height, color, filled } => {
            let img = load_image(&input)?;
            let params = RectangleParams {
                x, y, width, height, color, filled,
                stroke_width: 0,
                stroke_color: None,
                border_radius: 0,
            };
            let result = draw_rectangle(&img, &params);
            save_image(&result, &output)?;
            println!("Drew rectangle");
            Ok(())
        }

        Commands::DrawCircle { input, output, x, y, radius, color, filled } => {
            let img = load_image(&input)?;
            let params = CircleMaskParams {
                x, y, radius, color, filled,
                stroke_width: 0,
                stroke_color: None,
            };
            let result = draw_circle(&img, &params);
            save_image(&result, &output)?;
            println!("Drew circle");
            Ok(())
        }

        Commands::Canvas { output, width, height, color } => {
            let color_rgba = hex_to_rgba(&color).unwrap_or(Rgba([0, 0, 0, 255]));
            let mut canvas = RgbaImage::new(width, height);
            for px in canvas.pixels_mut() {
                *px = color_rgba;
            }
            let result = DynamicImage::ImageRgba8(canvas);
            save_image(&result, &output)?;
            println!("Created {}x{} canvas", width, height);
            Ok(())
        }

        Commands::Gradient { output, width, height, start, end, angle } => {
            let params = GradientParams {
                start_color: start,
                end_color: end,
                angle,
                width,
                height,
            };
            let result = create_gradient(&params);
            save_image(&result, &output)?;
            println!("Created {}x{} gradient", width, height);
            Ok(())
        }

        Commands::Mask { input, output, shape, x, y, width, height } => {
            let img = load_image(&input)?;
            let params = ShapeMaskParams {
                shape: shape.clone(), x, y, width, height,
                feather: false,
                feather_radius: 0,
            };
            let result = create_shape_mask(&img, &params);
            save_image(&result, &output)?;
            println!("Created {} mask", shape);
            Ok(())
        }

        Commands::Pipeline { input, output, config } => {
            let img = load_image(&input)?;
            let config_str = std::fs::read_to_string(&config)?;
            let ops: Vec<serde_json::Value> = serde_json::from_str(&config_str)?;
            let mut result = img;

            for op in &ops {
                result = apply_pipeline_op(result, op)?;
            }

            save_image(&result, &output)?;
            println!("Pipeline completed: {} operations", ops.len());
            Ok(())
        }
    }
}

fn apply_pipeline_op(img: DynamicImage, op: &serde_json::Value) -> anyhow::Result<DynamicImage> {
    let op_name = op["op"].as_str().ok_or_else(|| anyhow::anyhow!("Missing 'op' field"))?;

    match op_name {
        "crop" => Ok(crop(&img, &serde_json::from_value(op.clone())?)),
        "resize" => Ok(resize(&img, &serde_json::from_value(op.clone())?)),
        "rotate" => Ok(rotate(&img, &serde_json::from_value(op.clone())?)),
        "flip" => Ok(flip(&img, &serde_json::from_value(op.clone())?)),
        "blur" => Ok(blur(&img, &serde_json::from_value(op.clone())?)),
        "brightness" => Ok(brightness(&img, &serde_json::from_value(op.clone())?)),
        "contrast" => Ok(contrast(&img, &serde_json::from_value(op.clone())?)),
        "grayscale" => Ok(grayscale(&img, &serde_json::from_value(op.clone())?)),
        "tint" => Ok(tint(&img, &serde_json::from_value(op.clone())?)),
        "vignette" => Ok(vignette(&img, &serde_json::from_value(op.clone())?)),
        "duotone" => Ok(duotone(&img, &serde_json::from_value(op.clone())?)),
        "noise" => Ok(noise(&img, &serde_json::from_value(op.clone())?)),
        "liquify" => Ok(liquify(&img, &serde_json::from_value(op.clone())?)),
        "warp" => Ok(warp(&img, &serde_json::from_value(op.clone())?)),
        "glitch" => Ok(glitch(&img, &serde_json::from_value(op.clone())?)),
        "scanlines" => Ok(scanlines(&img, &serde_json::from_value(op.clone())?)),
        "halftone" => Ok(halftone(&img, &serde_json::from_value(op.clone())?)),
        "posterize" => Ok(posterize(&img, &serde_json::from_value(op.clone())?)),
        "edge_detect" => Ok(edge_detect(&img, &serde_json::from_value(op.clone())?)),
        "emboss" => Ok(emboss(&img, &serde_json::from_value(op.clone())?)),
        "solarize" => Ok(solarize(&img, &serde_json::from_value(op.clone())?)),
        "chromatic_aberration" => Ok(chromatic_aberration(&img, &serde_json::from_value(op.clone())?)),
        "pixel_sort" => Ok(pixel_sort(&img, &serde_json::from_value(op.clone())?)),
        "kaleidoscope" => Ok(kaleidoscope(&img, &serde_json::from_value(op.clone())?)),
        "text" | "draw_text" => Ok(draw_text(&img, &serde_json::from_value(op.clone())?)),
        "overlay" => {
            let overlay_path = op["overlay"].as_str().ok_or_else(|| anyhow::anyhow!("Missing 'overlay' field"))?;
            let top = load_image(&PathBuf::from(overlay_path))?;
            let params: OverlayParams = serde_json::from_value(op.clone())?;
            Ok(overlay(&img, &top, &params))
        }
        _ => Err(anyhow::anyhow!("Unknown operation: {}", op_name)),
    }
}
