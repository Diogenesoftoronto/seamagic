use base64::Engine;
use image::{DynamicImage, GenericImageView, ImageFormat};
use std::io::Cursor;

/// Maximum dimensions for an AI-visible preview.
/// Most vision models support images up to 1024x1024, but smaller
/// images fit better in context windows. We target 512 as a good
/// balance between detail and token economy.
pub const AI_PREVIEW_MAX_SIZE: u32 = 512;

/// Generate a compressed preview optimized for sending to an AI vision model.
/// The image is resized to fit within AI_PREVIEW_MAX_SIZE while preserving
/// aspect ratio, then encoded as JPEG at reduced quality for maximum
/// compression.
pub fn generate_ai_preview(img: &DynamicImage) -> anyhow::Result<(String, u32, u32)> {
    let (w, h) = img.dimensions();
    let (new_w, new_h) = if w.max(h) > AI_PREVIEW_MAX_SIZE {
        if w >= h {
            let ratio = AI_PREVIEW_MAX_SIZE as f32 / w as f32;
            (AI_PREVIEW_MAX_SIZE, (h as f32 * ratio) as u32)
        } else {
            let ratio = AI_PREVIEW_MAX_SIZE as f32 / h as f32;
            ((w as f32 * ratio) as u32, AI_PREVIEW_MAX_SIZE)
        }
    } else {
        (w, h)
    };

    let preview = img.resize(new_w, new_h, image::imageops::FilterType::Lanczos3);

    let mut buffer = Vec::new();
    {
        let mut cursor = Cursor::new(&mut buffer);
        preview.write_to(&mut cursor, ImageFormat::Jpeg)?;
    }

    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &buffer);
    Ok((b64, new_w, new_h))
}

/// Generate a compressed preview and return it as a structured result
/// suitable for MCP tool responses.
pub fn generate_preview_result(img: &DynamicImage) -> anyhow::Result<crate::mcp::protocol::PreviewResult> {
    let (data, width, height) = generate_ai_preview(img)?;
    Ok(crate::mcp::protocol::PreviewResult {
        data,
        mime_type: "image/jpeg".to_string(),
        width,
        height,
    })
}

/// Create a side-by-side comparison image and also generate a compressed
/// preview for AI visibility.
pub fn create_comparison_with_preview(
    original: &DynamicImage,
    edited: &DynamicImage,
    params: &super::composite::ComparisonParams,
) -> anyhow::Result<(DynamicImage, crate::mcp::protocol::PreviewResult)> {
    let comparison = super::composite::create_comparison(original, edited, params);
    let preview = generate_preview_result(&comparison)?;
    Ok((comparison, preview))
}

/// Create a pixel diff and also generate a compressed preview.
pub fn create_diff_with_preview(
    original: &DynamicImage,
    edited: &DynamicImage,
    params: &super::composite::DiffParams,
) -> anyhow::Result<(DynamicImage, crate::mcp::protocol::PreviewResult)> {
    let diff = super::composite::create_diff(original, edited, params);
    let preview = generate_preview_result(&diff)?;
    Ok((diff, preview))
}
