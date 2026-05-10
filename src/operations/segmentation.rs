use base64::Engine;
use image::{DynamicImage, ImageFormat};
use std::io::Cursor;

/// Replicate SAM-2 automatic mask generation parameters.
#[derive(Debug, Clone)]
pub struct SegmentImageParams {
    pub image_b64: String,
    pub api_key: Option<String>,
    pub model: String,
}

/// Background removal via Replicate RemoveBG model.
#[derive(Debug, Clone)]
pub struct RemoveBackgroundParams {
    pub image_b64: String,
    pub api_key: Option<String>,
}

impl Default for SegmentImageParams {
    fn default() -> Self {
        Self {
            image_b64: String::new(),
            api_key: None,
            model: "meta/sam-2:fe97b453a6455861e3bac769b441ca1f1086110da7466dbb65cf1eecfd60dc83".to_string(),
        }
    }
}

fn get_replicate_key(api_key: &Option<String>) -> anyhow::Result<String> {
    api_key
        .clone()
        .or_else(|| std::env::var("REPLICATE_API_TOKEN").ok())
        .ok_or_else(|| anyhow::anyhow!("Replicate API token required. Set REPLICATE_API_TOKEN environment variable or pass api_key."))
}

fn img_to_b64(img: &DynamicImage) -> anyhow::Result<String> {
    let mut buf = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&buf))
}

/// Call Replicate's prediction API and poll for completion.
async fn replicate_predict(
    api_key: &str,
    model: &str,
    input: serde_json::Value,
) -> anyhow::Result<serde_json::Value> {
    let client = reqwest::Client::new();
    let parts: Vec<&str> = model.split(':').collect();
    let (model_path, version) = if parts.len() == 2 {
        (parts[0], Some(parts[1]))
    } else {
        (model, None)
    };

    let body = if let Some(v) = version {
        serde_json::json!({
            "version": v,
            "input": input,
        })
    } else {
        serde_json::json!({
            "input": input,
        })
    };

    let resp = client
        .post(&format!("https://api.replicate.com/v1/models/{}/predictions", model_path))
        .header("Authorization", format!("Token {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await?;
        return Err(anyhow::anyhow!(
            "Replicate prediction failed for model '{}'. POST returned {}: {}. Make sure REPLICATE_API_TOKEN is set and the model is available.",
            model_path, status, text
        ));
    }

    let initial: serde_json::Value = resp.json().await?;
    let prediction_url = initial["urls"]["get"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No prediction URL in Replicate response"))?
        .to_string();

    // Poll for completion (max 5 minutes)
    let start = std::time::Instant::now();
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        let status_resp = client
            .get(&prediction_url)
            .header("Authorization", format!("Token {}", api_key))
            .send()
            .await?;

        let poll_status = status_resp.status();
        if !poll_status.is_success() {
            let text = status_resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Replicate status check failed. GET {} returned {}: {}. Model may be unavailable or your API key is invalid.",
                prediction_url, poll_status, text
            ));
        }

        let status: serde_json::Value = status_resp.json().await?;
        let status_str = status["status"].as_str().unwrap_or("unknown");

        match status_str {
            "succeeded" => return Ok(status),
            "failed" | "canceled" => {
                return Err(anyhow::anyhow!(
                    "Replicate prediction {} for model '{}'. Error: {}. Full status: {}",
                    status_str, model_path,
                    status["error"].as_str().unwrap_or("unknown error"),
                    serde_json::to_string_pretty(&status).unwrap_or_default()
                ));
            }
            _ => {
                if start.elapsed().as_secs() > 300 {
                    return Err(anyhow::anyhow!("Replicate prediction timed out after 5 minutes"));
                }
                continue;
            }
        }
    }
}

/// Download an image from a URL and return as DynamicImage.
async fn download_image(url: &str) -> anyhow::Result<DynamicImage> {
    let client = reqwest::Client::new();
    let resp = client.get(url).send().await?;
    let status = resp.status();
    if !status.is_success() {
        return Err(anyhow::anyhow!(
            "Failed to download image from {}: HTTP {}. If this is a Replicate output URL, the result may have expired (URLs are temporary).",
            url, status
        ));
    }
    let bytes = resp.bytes().await?;
    let img = image::load_from_memory(&bytes).map_err(|e| {
        anyhow::anyhow!(
            "Downloaded {} bytes from {}, but failed to decode as image: {}. The URL may not point to a valid image file.",
            bytes.len(), url, e
        )
    })?;
    Ok(img)
}

/// Segment everything in an image using Replicate SAM-2.
/// Returns: (combined_mask_image, Vec<individual_mask_images>)
pub async fn segment_images(params: &SegmentImageParams) -> anyhow::Result<(DynamicImage, Vec<DynamicImage>)> {
    let api_key = get_replicate_key(&params.api_key)?;

    let input = serde_json::json!({
        "image": format!("data:image/png;base64,{}", params.image_b64),
    });

    let result = replicate_predict(&api_key, &params.model, input).await?;

    let combined_url = result["output"]["combined_mask"]
        .as_str()
        .ok_or_else(|| {
            let out = serde_json::to_string_pretty(&result["output"]).unwrap_or_else(|_| "<unprintable>".to_string());
            anyhow::anyhow!("No combined_mask in Replicate output: {}", out)
        })?;

    let combined = download_image(combined_url).await?;

    let individual_urls: Vec<String> = result["output"]["individual_masks"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    let mut individuals = Vec::new();
    for url in &individual_urls {
        match download_image(url).await {
            Ok(img) => individuals.push(img),
            Err(e) => eprintln!("Warning: failed to download individual mask: {}", e),
        }
    }

    Ok((combined, individuals))
}

/// Get individual mask PNGs as base64 strings.
pub async fn segment_images_to_b64(params: &SegmentImageParams) -> anyhow::Result<(String, Vec<String>)> {
    let (combined, individuals) = segment_images(params).await?;
    let combined_b64 = img_to_b64(&combined)?;
    let mut masks_b64 = Vec::new();
    for img in &individuals {
        masks_b64.push(img_to_b64(img)?);
    }
    Ok((combined_b64, masks_b64))
}

/// Remove background using Replicate remove-bg model.
pub async fn remove_background(params: &RemoveBackgroundParams) -> anyhow::Result<DynamicImage> {
    let api_key = get_replicate_key(&params.api_key)?;

    let input = serde_json::json!({
        "image": format!("data:image/png;base64,{}", params.image_b64),
    });

    let result = replicate_predict(
        &api_key,
        "lucataco/remove-bg:95fcc2a26d3899cd6a2699c0bdc05299112b19712222a4b62",
        input,
    ).await?;

    let output = &result["output"];
    let url = output.as_str().or_else(|| output["image"].as_str())
        .ok_or_else(|| {
            let out_str = serde_json::to_string_pretty(&output).unwrap_or_else(|_| "<unprintable>".to_string());
            anyhow::anyhow!(
                "Replicate remove-bg completed but returned no image URL. Output: {}. Model: lucataco/remove-bg. Ensure the input image is valid.",
                out_str
            )
        })?;

    download_image(url).await
}

/// Remove background and return base64 PNG.
pub async fn remove_background_to_b64(params: &RemoveBackgroundParams) -> anyhow::Result<String> {
    let img = remove_background(params).await?;
    img_to_b64(&img)
}
