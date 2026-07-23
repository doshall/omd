use base64::Engine;
use image::ImageEncoder;

/// Read plain text from the system clipboard.
pub fn clipboard_text() -> Option<String> {
    let mut clipboard = arboard::Clipboard::new().ok()?;
    clipboard.get_text().ok()
}

/// Write plain text to the system clipboard.
pub fn set_clipboard_text(text: &str) -> bool {
    arboard::Clipboard::new()
        .and_then(|mut c| c.set_text(text.to_string()).map(|_| ()))
        .is_ok()
}

/// Read an image from the system clipboard and return a PNG data URL.
pub fn clipboard_image_data_url() -> Option<String> {
    let mut clipboard = arboard::Clipboard::new().ok()?;
    let image = clipboard.get_image().ok()?;
    let width = image.width as u32;
    let height = image.height as u32;
    let rgba = image.bytes.into_owned();

    let img = image::RgbaImage::from_raw(width, height, rgba)?;
    let mut png_bytes = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
    encoder
        .write_image(
            img.as_raw(),
            width,
            height,
            image::ExtendedColorType::Rgba8,
        )
        .ok()?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);
    Some(format!("data:image/png;base64,{b64}"))
}
