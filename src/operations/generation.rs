use base64::Engine;
use image::{DynamicImage, ImageFormat};
use std::io::Cursor;

/// Parameters for generating an image via an external AI service (OpenAI DALL-E).
#[derive(Debug, Clone)]
pub struct GenerateImageParams {
    pub prompt: String,
    pub width: u32,
    pub height: u32,
    pub model: String, // "dall-e-3" or "dall-e-2"
    pub quality: String, // "standard" or "hd"
    pub style: String, // "vivid" or "natural"
    pub api_key: Option<String>,
}

/// Parameters for filling a masked region with AI-generated content (OpenAI inpainting).
#[derive(Debug, Clone)]
pub struct FillWithGenerationParams {
    pub prompt: String,
    pub mask_image_b64: String,
    pub original_image_b64: String,
    pub model: String,
    pub api_key: Option<String>,
}

/// Default parameters for a YouTube thumbnail generation.
impl Default for GenerateImageParams {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            width: 1024,
            height: 1024,
            model: "dall-e-3".to_string(),
            quality: "standard".to_string(),
            style: "vivid".to_string(),
            api_key: None,
        }
    }
}

/// Generate an image using OpenAI's DALL-E API.
/// Requires an OPENAI_API_KEY environment variable or api_key in params.
pub async fn generate_image(params: &GenerateImageParams) -> anyhow::Result<DynamicImage> {
    let api_key = params
        .api_key
        .clone()
        .or_else(|| std::env::var("OPENAI_API_KEY").ok())
        .ok_or_else(|| anyhow::anyhow!("OpenAI API key required. Set OPENAI_API_KEY environment variable or pass api_key."))?;

    let size = match (params.width, params.height) {
        (1024, 1792) => "1024x1792",
        (1792, 1024) => "1792x1024",
        (1024, 1024) => "1024x1024",
        (512, 512) => "512x512",
        (256, 256) => "256x256",
        _ => "1024x1024",
    };

    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": params.model,
        "prompt": params.prompt,
        "n": 1,
        "size": size,
        "quality": params.quality,
        "style": params.style,
        "response_format": "b64_json",
    });

    let response = client
        .post("https://api.openai.com/v1/images/generations")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        let text = response.text().await?;
        return Err(anyhow::anyhow!(
            "OpenAI image generation failed: POST /v1/images/generations returned {}: {}. Ensure OPENAI_API_KEY is set and valid. (key prefix: {}...)",
            status, text,
            &api_key[..api_key.len().min(4)]
        ));
    }

    let json: serde_json::Value = response.json().await?;
    let b64 = json["data"][0]["b64_json"]
        .as_str()
        .ok_or_else(|| {
            let resp_str = serde_json::to_string_pretty(&json).unwrap_or_else(|_| "<unprintable>".to_string());
            anyhow::anyhow!(
                "OpenAI generation succeeded (HTTP 200) but response contained no base64 image data. Response: {}. The prompt may have been rejected by content policy.",
                resp_str
            )
        })?;

    let bytes = base64::engine::general_purpose::STANDARD.decode(b64)?;
    let img = image::load_from_memory(&bytes)?;

    // Resize to exact requested dimensions if they differ from DALL-E output
    let (w, h) = (params.width, params.height);
    if w != img.width() || h != img.height() {
        Ok(img.resize_exact(w, h, image::imageops::FilterType::Lanczos3))
    } else {
        Ok(img)
    }
}

/// Fill a masked region with AI-generated content using OpenAI's GPT-Image-1 / DALL-E edit API.
/// This creates a new image where the masked area is replaced with generated content.
pub async fn fill_with_generation(params: &FillWithGenerationParams) -> anyhow::Result<DynamicImage> {
    let api_key = params
        .api_key
        .clone()
        .or_else(|| std::env::var("OPENAI_API_KEY").ok())
        .ok_or_else(|| anyhow::anyhow!("OpenAI API key required. Set OPENAI_API_KEY environment variable or pass api_key."))?;

    let client = reqwest::Client::new();

    // Decode the base64 images to get bytes for multipart upload
    let original_bytes = base64::engine::general_purpose::STANDARD.decode(&params.original_image_b64)?;
    let mask_bytes = base64::engine::general_purpose::STANDARD.decode(&params.mask_image_b64)?;

    let form = reqwest::multipart::Form::new()
        .text("model", params.model.clone())
        .text("prompt", params.prompt.clone())
        .text("n", "1")
        .text("size", "1024x1024")
        .text("response_format", "b64_json")
        .part("image", reqwest::multipart::Part::bytes(original_bytes).file_name("image.png").mime_str("image/png")?)
        .part("mask", reqwest::multipart::Part::bytes(mask_bytes).file_name("mask.png").mime_str("image/png")?);

    let response = client
        .post("https://api.openai.com/v1/images/edits")
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        let text = response.text().await?;
        return Err(anyhow::anyhow!(
            "OpenAI image edit failed: POST /v1/images/edits returned {}: {}. Ensure OPENAI_API_KEY is valid and the image/mask are valid PNGs under 4MB.",
            status, text
        ));
    }

    let json: serde_json::Value = response.json().await?;
    let b64 = json["data"][0]["b64_json"]
        .as_str()
        .ok_or_else(|| {
            let resp_str = serde_json::to_string_pretty(&json).unwrap_or_else(|_| "<unprintable>".to_string());
            anyhow::anyhow!(
                "OpenAI edit succeeded (HTTP 200) but response contained no base64 image data. Response: {}. Check if the masked region was rejected by content policy.",
                resp_str
            )
        })?;

    let bytes = base64::engine::general_purpose::STANDARD.decode(b64)?;
    let img = image::load_from_memory(&bytes)?;
    Ok(img)
}

/// Parameters for editing an image via fal.ai openai/gpt-image-2/edit.
#[derive(Debug, Clone)]
pub struct FalEditParams {
    pub prompt: String,
    pub image_b64: String,
    pub mask_b64: Option<String>,
    pub quality: String,      // auto, low, medium, high
    pub num_images: u32,
    pub output_format: String, // png, jpeg, webp
    pub image_size: String,   // auto, square_hd, square, etc.
    pub api_key: Option<String>,
}

impl Default for FalEditParams {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            image_b64: String::new(),
            mask_b64: None,
            quality: "high".to_string(),
            num_images: 1,
            output_format: "png".to_string(),
            image_size: "auto".to_string(),
            api_key: None,
        }
    }
}

fn get_fal_key(api_key: &Option<String>) -> anyhow::Result<String> {
    api_key.clone().or_else(|| {
        let var = std::env::var("FAL_KEY").ok();
        if var.is_none() {
            eprintln!("[fal.ai] FAL_KEY is not set in environment");
        }
        var
    }).ok_or_else(|| anyhow::anyhow!(
        "FAL_KEY required for fal.ai GPT Image 2. Set FAL_KEY environment variable or pass api_key parameter. Get a key at https://fal.ai/dashboard"
    ))
}

/// Edit an image using fal.ai openai/gpt-image-2/edit endpoint.
/// Supports mask-based inpainting and natural language editing.
pub async fn fal_edit_image(params: &FalEditParams) -> anyhow::Result<DynamicImage> {
    let api_key = get_fal_key(&params.api_key)?;
    let client = reqwest::Client::new();

    let image_url = format!("data:image/png;base64,{}", params.image_b64);
    let mask_url = params.mask_b64.as_ref().map(|m| format!("data:image/png;base64,{}", m));

    let mut body = serde_json::json!({
        "prompt": params.prompt,
        "image_urls": [image_url],
        "quality": params.quality,
        "num_images": params.num_images,
        "output_format": params.output_format,
        "image_size": params.image_size,
    });

    if let Some(ref mask) = mask_url {
        body["mask_url"] = serde_json::Value::String(mask.clone());
    }

    // Submit request
    let submit_resp = client
        .post("https://queue.fal.run/openai/gpt-image-2/edit")
        .header("Authorization", format!("Key {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    let submit_status = submit_resp.status();
    if !submit_status.is_success() {
        let text = submit_resp.text().await?;
        return Err(anyhow::anyhow!(
            "fal.ai submit failed for openai/gpt-image-2/edit. POST https://queue.fal.run/openai/gpt-image-2/edit returned {}: {}. Make sure FAL_KEY is set and valid. (key prefix: {}...)",
            submit_status, text,
            &api_key[..api_key.len().min(4)]
        ));
    }

    let status: serde_json::Value = submit_resp.json().await?;
    let request_id = status["request_id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No request_id in fal.ai response"))?
        .to_string();

    // Poll for completion
    let start = std::time::Instant::now();
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let status_resp = client
            .get(&format!("https://queue.fal.run/openai/gpt-image-2/edit/requests/{}/status", request_id))
            .header("Authorization", format!("Key {}", api_key))
            .send()
            .await?;

        let poll_status = status_resp.status();
        if !poll_status.is_success() {
            let text = status_resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "fal.ai status poll failed for request_id '{}': GET returned {}: {}. The request may have been rejected or the key is invalid.",
                request_id, poll_status, text
            ));
        }

        let status_json: serde_json::Value = status_resp.json().await?;
        let status_str = status_json["status"].as_str().unwrap_or("unknown");

        match status_str {
            "COMPLETED" => {
                // Fetch result
                let result_resp = client
                    .get(&format!("https://queue.fal.run/openai/gpt-image-2/edit/requests/{}", request_id))
                    .header("Authorization", format!("Key {}", api_key))
                    .send()
                    .await?;

                let result_status = result_resp.status();
                if !result_status.is_success() {
                    let text = result_resp.text().await?;
                    return Err(anyhow::anyhow!(
                        "fal.ai result fetch failed for request_id '{}': GET returned {}: {}",
                        request_id, result_status, text
                    ));
                }

                let result: serde_json::Value = result_resp.json().await?;
                let result_str = serde_json::to_string_pretty(&result).unwrap_or_else(|_| "<unprintable>".to_string());
                let img_url = result["images"][0]["url"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!(
                        "fal.ai completed but returned no image URL for request_id '{}'. Response: {}. Check if the prompt was blocked or the generation failed server-side.",
                        request_id, result_str
                    ))?;

                // Download image
                let img_resp = client.get(img_url).send().await?;
                let dl_status = img_resp.status();
                if !dl_status.is_success() {
                    return Err(anyhow::anyhow!(
                        "fal.ai returned image URL but download failed: GET {} returned HTTP {}. The generated image may have expired. request_id: {}",
                        img_url, dl_status, request_id
                    ));
                }
                let bytes = img_resp.bytes().await?;
                return image::load_from_memory(&bytes).map_err(|e| e.into());
            }
            "IN_PROGRESS" | "IN_QUEUE" => {
                if start.elapsed().as_secs() > 300 {
                    return Err(anyhow::anyhow!("fal.ai edit timed out after 5 minutes"));
                }
                continue;
            }
            _ => {
                let status_dump = serde_json::to_string_pretty(&status_json).unwrap_or_default();
                return Err(anyhow::anyhow!(
                    "fal.ai edit got unexpected status '{}' for request_id '{}'. Response: {}. This may indicate a transient error — retry may help.",
                    status_str, request_id, status_dump
                ));
            }
        }
    }
}

/// Edit image and return base64 PNG.
pub async fn fal_edit_image_to_b64(params: &FalEditParams) -> anyhow::Result<String> {
    let img = fal_edit_image(params).await?;
    let mut buffer = Vec::new();
    img.write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)?;
    Ok(base64::engine::general_purpose::STANDARD.encode(buffer))
}

/// Convenience: generate an image and return it as base64 PNG.
pub async fn generate_image_to_b64(params: &GenerateImageParams) -> anyhow::Result<String> {
    let img = generate_image(params).await?;
    let mut buffer = Vec::new();
    img.write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)?;
    Ok(base64::engine::general_purpose::STANDARD.encode(buffer))
}
