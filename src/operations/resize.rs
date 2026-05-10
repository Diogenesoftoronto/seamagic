use image::{DynamicImage, GenericImageView, imageops::FilterType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ResizeMode {
    Exact,
    Fit,
    Fill,
    Scale,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResizeParams {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub mode: ResizeMode,
    #[serde(default = "default_filter")]
    pub filter: String,
}

fn default_filter() -> String {
    "lanczos3".to_string()
}

fn parse_filter(filter: &str) -> FilterType {
    match filter.to_lowercase().as_str() {
        "nearest" => FilterType::Nearest,
        "triangle" => FilterType::Triangle,
        "catmullrom" => FilterType::CatmullRom,
        "gaussian" => FilterType::Gaussian,
        "lanczos3" | _ => FilterType::Lanczos3,
    }
}

pub fn resize(img: &DynamicImage, params: &ResizeParams) -> DynamicImage {
    let filter = parse_filter(&params.filter);
    let (orig_w, orig_h) = img.dimensions();

    let (new_w, new_h) = match params.mode {
        ResizeMode::Exact => {
            let w = params.width.unwrap_or(orig_w);
            let h = params.height.unwrap_or(orig_h);
            (w, h)
        }
        ResizeMode::Fit => {
            let target_w = params.width.unwrap_or(orig_w);
            let target_h = params.height.unwrap_or(orig_h);
            let ratio = (target_w as f32 / orig_w as f32)
                .min(target_h as f32 / orig_h as f32);
            ((orig_w as f32 * ratio) as u32, (orig_h as f32 * ratio) as u32)
        }
        ResizeMode::Fill => {
            let target_w = params.width.unwrap_or(orig_w);
            let target_h = params.height.unwrap_or(orig_h);
            let ratio = (target_w as f32 / orig_w as f32)
                .max(target_h as f32 / orig_h as f32);
            ((orig_w as f32 * ratio) as u32, (orig_h as f32 * ratio) as u32)
        }
        ResizeMode::Scale => {
            let scale_w = params.width.map(|w| w as f32 / orig_w as f32).unwrap_or(1.0);
            let scale_h = params.height.map(|h| h as f32 / orig_h as f32).unwrap_or(scale_w);
            ((orig_w as f32 * scale_w) as u32, (orig_h as f32 * scale_h) as u32)
        }
    };

    img.resize(new_w.max(1), new_h.max(1), filter)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailParams {
    pub size: u32,
}

pub fn thumbnail(img: &DynamicImage, params: &ThumbnailParams) -> DynamicImage {
    img.thumbnail(params.size, params.size)
}
