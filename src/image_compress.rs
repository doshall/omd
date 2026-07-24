use base64::Engine;
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use image::{ColorType, GenericImageView, ImageEncoder};

pub fn compress_data_url(data_url: &str, max_width: u32, quality: u8) -> Option<String> {
    let (mime, data) = parse_data_url(data_url)?;
    if mime.contains("svg") {
        return Some(data_url.to_string());
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data)
        .ok()?;
    let compressed = compress_image_bytes(&bytes, max_width, quality)?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(&compressed);
    Some(format!("data:image/jpeg;base64,{encoded}"))
}

pub fn compress_image_bytes(bytes: &[u8], max_width: u32, quality: u8) -> Option<Vec<u8>> {
    let img = image::load_from_memory(bytes).ok()?;
    let (w, h) = img.dimensions();
    let img = if w > max_width {
        let nh = (h as f64 * f64::from(max_width) / f64::from(w)).round() as u32;
        img.resize(max_width, nh.max(1), FilterType::Lanczos3)
    } else {
        img
    };
    let rgba = img.to_rgba8();
    let rgb = image::DynamicImage::ImageRgba8(rgba).into_rgb8();
    let (width, height) = rgb.dimensions();
    let mut out = Vec::new();
    let quality = quality.clamp(1, 100);
    let encoder = JpegEncoder::new_with_quality(&mut out, quality);
    encoder
        .write_image(rgb.as_raw(), width, height, ColorType::Rgb8.into())
        .ok()?;
    Some(out)
}

fn parse_data_url(data_url: &str) -> Option<(&str, &str)> {
    let data_url = data_url.trim();
    let rest = data_url.strip_prefix("data:")?;
    let (meta, data) = rest.split_once(',')?;
    if !meta.ends_with(";base64") {
        return None;
    }
    let mime = meta.strip_suffix(";base64")?;
    Some((mime, data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compresses_png_to_jpeg_data_url() {
        let mut png = Vec::new();
        let img = image::RgbaImage::from_pixel(2000, 100, image::Rgba([10, 20, 30, 255]));
        img.write_to(
            &mut std::io::Cursor::new(&mut png),
            image::ImageFormat::Png,
        )
        .unwrap();
        let b64 = base64::engine::general_purpose::STANDARD.encode(&png);
        let data_url = format!("data:image/png;base64,{b64}");
        let out = compress_data_url(&data_url, 800, 80).unwrap();
        assert!(out.starts_with("data:image/jpeg;base64,"));
    }
}
