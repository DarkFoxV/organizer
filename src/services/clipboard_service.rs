use arboard::{Clipboard, ImageData};
use std::sync::{Mutex, OnceLock};

static CLIPBOARD: OnceLock<Mutex<Clipboard>> = OnceLock::new();

fn get_clipboard() -> &'static Mutex<Clipboard> {
    CLIPBOARD.get_or_init(|| {
        Mutex::new(Clipboard::new().expect("Failed to create Clipboard"))
    })
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

