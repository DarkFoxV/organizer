use arboard::{Clipboard, ImageData};
use image::DynamicImage;
use log::info;
use std::sync::{Mutex, OnceLock};
use crate::services::file_service::detect_image_format;

static CLIPBOARD: OnceLock<Mutex<Clipboard>> = OnceLock::new();

pub fn get_clipboard() -> &'static Mutex<Clipboard> {
    CLIPBOARD.get_or_init(|| Mutex::new(Clipboard::new().expect("Failed to create Clipboard")))
}

pub fn copy_image_to_clipboard(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let img = image::open(path)?.to_rgba8();
    let (width, height) = img.dimensions();

    let img_data = ImageData {
        width: width as usize,
        height: height as usize,
        bytes: img.into_raw().into(),
    };

    let clipboard = get_clipboard();
    let mut clipboard = clipboard.lock().unwrap();
    clipboard.set_image(img_data)?;

    Ok(())
}

fn get_direct_image(clipboard: &mut Clipboard) -> Option<(DynamicImage, image::ImageFormat)> {
    match clipboard.get_image() {
        Ok(image_data) => {
            info!("It's an image from clipboard");

            let dynamic_image = DynamicImage::ImageRgba8(
                image::ImageBuffer::from_raw(
                    image_data.width as u32,
                    image_data.height as u32,
                    image_data.bytes.to_vec(),
                )
                    .expect("Failed to create ImageBuffer from clipboard data"),
            );

            Some((dynamic_image, image::ImageFormat::Png))
        }
        Err(_) => None,
    }
}

fn load_image_from_path(path: &std::path::Path) -> Option<(DynamicImage, image::ImageFormat)> {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            info!("Failed to read file from path: {}", e);
            return None;
        }
    };

    let kind = match infer::get(&bytes) {
        Some(k) => k,
        None => {
            info!("Could not detect file type from path");
            return None;
        }
    };

    if !kind.mime_type().starts_with("image/") {
        info!("File is not an image ({})", kind.mime_type());
        return None;
    }

    info!("File is an image ({}), loading...", kind.mime_type());
    let format = detect_image_format(&bytes);

    match image::load_from_memory_with_format(&bytes, format) {
        Ok(loaded_image) => {
            info!("Image successfully loaded from path with format: {:?}", format);
            Some((loaded_image, format))
        }
        Err(e) => {
            info!("Failed to decode image from path: {}", e);
            None
        }
    }
}

fn get_image_from_text_path(clipboard: &mut Clipboard) -> Option<(DynamicImage, image::ImageFormat)> {
    info!("Not an image, trying text/path");

    let clipboard_text = match clipboard.get_text() {
        Ok(text) => text,
        Err(_) => {
            info!("Failed to get text from clipboard");
            return None;
        }
    };

    let trimmed_path = clipboard_text.trim();
    let path = std::path::Path::new(trimmed_path);

    if !path.exists() || !path.is_file() {
        info!("Clipboard text is not a valid file path: {}", trimmed_path);
        return None;
    }

    info!("Clipboard contains a file path: {}", trimmed_path);
    load_image_from_path(path)
}

/// Method to get the image from the clipboard
pub fn get_clipboard_image() -> Option<(DynamicImage, image::ImageFormat)> {
    let clipboard = get_clipboard();

    let mut clipboard_lock = match clipboard.lock() {
        Ok(lock) => lock,
        Err(_) => return None,
    };

    if let Some(result) = get_direct_image(&mut clipboard_lock) {
        return Some(result);
    }

    get_image_from_text_path(&mut clipboard_lock)
}

