use image::{DynamicImage, GenericImageView};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CropParams {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CropFromCenterParams {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmartCropParams {
    pub aspect_ratio: Option<f32>,
    pub target_width: Option<u32>,
    pub target_height: Option<u32>,
    pub focus_x: Option<f32>,
    pub focus_y: Option<f32>,
}

pub fn crop(img: &DynamicImage, params: &CropParams) -> DynamicImage {
    img.crop_imm(params.x, params.y, params.width, params.height)
}

pub fn crop_from_center(img: &DynamicImage, params: &CropFromCenterParams) -> DynamicImage {
    let (img_w, img_h) = img.dimensions();
    let x = img_w.saturating_sub(params.width) / 2;
    let y = img_h.saturating_sub(params.height) / 2;
    let w = params.width.min(img_w);
    let h = params.height.min(img_h);
    img.crop_imm(x, y, w, h)
}

pub fn smart_crop(img: &DynamicImage, params: &SmartCropParams) -> DynamicImage {
    let (img_w, img_h) = img.dimensions();
    let img_ratio = img_w as f32 / img_h as f32;

    let target_ratio = params.aspect_ratio.unwrap_or(img_ratio);

    let focus_x = params.focus_x.unwrap_or(0.5);
    let focus_y = params.focus_y.unwrap_or(0.5);

    let mut crop_w = img_w;
    let mut crop_h = img_h;

    if img_ratio > target_ratio {
        crop_w = (img_h as f32 * target_ratio) as u32;
    } else {
        crop_h = (img_w as f32 / target_ratio) as u32;
    }

    crop_w = crop_w.min(img_w);
    crop_h = crop_h.min(img_h);

    let x = ((img_w - crop_w) as f32 * focus_x) as u32;
    let y = ((img_h - crop_h) as f32 * focus_y) as u32;

    img.crop_imm(x, y, crop_w, crop_h)
}
