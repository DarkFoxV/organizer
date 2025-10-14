use crate::services::{file_service, thumbnail_service};
use arboard::{Clipboard, ImageData};
use image::DynamicImage;
use log::info;
use std::sync::{Mutex, OnceLock};

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

// Method to get the image from the clipboard
pub fn get_clipboard_image() -> Option<DynamicImage> {
    let clipboard = get_clipboard();
    let mut dynamic_image: Option<DynamicImage> = None;

    if let Ok(mut clipboard_lock) = clipboard.lock() {
        match clipboard_lock.get_image() {
            Ok(image_data) => {
                info!("It's an image");

                dynamic_image = Some(DynamicImage::ImageRgba8(
                    image::ImageBuffer::from_raw(
                        image_data.width as u32,
                        image_data.height as u32,
                        image_data.bytes.to_vec(),
                    )
                    .expect("Failed to create ImageBuffer from clipboard data"),
                ));
            }
            Err(_) => {
                info!("Not an image, trying text");

                if let Ok(clipboard_text) = clipboard_lock.get_text() {
                    if file_service::is_image_path(&clipboard_text) {
                        info!("String is a path to an image: {}", clipboard_text);

                        match thumbnail_service::open_image(&clipboard_text) {
                            Ok(loaded_image) => {
                                info!("Image successfully loaded from path");
                                dynamic_image = Some(loaded_image);
                            }
                            Err(e) => {
                                info!("Failed to load image from path: {}", e);
                            }
                        }
                    } else {
                        info!("String is not a valid image path");
                    }
                } else {
                    info!("Failed to get text from clipboard");
                }
            }
        }
    }
    dynamic_image
}
