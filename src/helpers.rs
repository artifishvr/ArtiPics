use base64::{Engine as _, engine::general_purpose};
use image::imageops::{blur, crop_imm, flip_vertical, overlay, resize};
use image::{DynamicImage, ImageReader, imageops::FilterType};
use serde::Deserialize;
use std::env;
use std::io::Cursor;

use crate::reqwest_client::get_client;

#[derive(Debug, Deserialize)]
struct DirectusResponse<T> {
    data: Vec<T>,
}

#[derive(Debug, Deserialize)]
struct MilfPicsItem {
    picture: String,
}

pub async fn get_images() -> Result<Vec<String>, reqwest::Error> {
    let url = env::var("DIRECTUS_URL").expect("DIRECTUS_URL not set");
    let token = env::var("DIRECTUS_TOKEN").expect("DIRECTUS_TOKEN not set");

    let response = get_client()
        .get(format!("{url}/items/thewall"))
        .bearer_auth(token)
        .query(&[("filter[hidden]", "false")])
        .query(&[("sort", "date_created")])
        .send()
        .await?
        .error_for_status()?;

    let text = response.text().await?;

    let response_data : DirectusResponse<MilfPicsItem> =
    serde_json::from_str(&text).unwrap();

    let pic_urls: Vec<String> = response_data
        .data
        .iter()
        .map(|item| format!("{url}/assets/{}?download", item.picture))
        .collect();

    Ok(pic_urls)
}

pub fn download_image_to_rgb_b64(
    url: &str,
    target_w: u32,
    target_h: u32,
) -> std::io::Result<String> {
    let response = reqwest::blocking::Client::new().get(url).send().unwrap();
    let bytes = response.bytes().unwrap();
    let og_img = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()?
        .decode()
        .unwrap();

    let img = flip_vertical(&og_img);

    let (src_w, src_h) = img.dimensions();

    let scale_fill = f64::max(
        target_w as f64 / src_w as f64,
        target_h as f64 / src_h as f64,
    );
    let scale_fit = f64::min(
        target_w as f64 / src_w as f64,
        target_h as f64 / src_h as f64,
    );

    // blurred bg
    let fill_w = (src_w as f64 * scale_fill).ceil() as u32;
    let fill_h = (src_h as f64 * scale_fill).ceil() as u32;
    let resized_bg = resize(&img, fill_w, fill_h, FilterType::CatmullRom);
    let bx: u32 = (fill_w.saturating_sub(target_w)) / 2;
    let by = (fill_h.saturating_sub(target_h)) / 2;
    let mut canvas = crop_imm(&resized_bg, bx, by, target_w, target_h).to_image();
    canvas = blur(&canvas, 40.0);

    // foreground
    let scaled_w = (src_w as f64 * scale_fit).floor() as u32;
    let scaled_h = (src_h as f64 * scale_fit).floor() as u32;
    let fg = resize(&img, scaled_w, scaled_h, FilterType::CatmullRom);
    let x = (target_w.saturating_sub(scaled_w)) / 2;
    let y = (target_h.saturating_sub(scaled_h)) / 2;
    overlay(&mut canvas, &fg, x as i64, y as i64);

    let rgb = DynamicImage::ImageRgba8(canvas).to_rgb8();

    let final_base64 = general_purpose::STANDARD.encode(rgb.into_raw());

    Ok(final_base64)
}
