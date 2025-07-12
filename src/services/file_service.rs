use crate::services::thumbnail_service::{generate_thumbnail, open_and_fix_image, save_corrected_image};
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;
use log::{error};

pub fn save_image_file_with_thumbnail(
    id: i64,
    original_path: &str,
) -> io::Result<(String, String)> {
    let images_dir = Path::new("images");
    if !images_dir.exists() {
        fs::create_dir(images_dir)?;
    }

    let image_dir = images_dir.join(id.to_string());
    if !image_dir.exists() {
        fs::create_dir(&image_dir)?;
    }

    let original_path = Path::new(original_path);
    let filename = original_path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid filename"))?;

    // Try to open and fix the image if necessary
    match open_and_fix_image(original_path) {
        Ok(img) => {
            // Image was opened (possibly corrected)
            let filename_str = filename.to_string_lossy();
            let stem = filename_str.split('.').next().unwrap_or(&filename_str);

            // Save corrected image as PNG
            let corrected_filename = format!("{}.png", stem);
            let corrected_path = image_dir.join(corrected_filename);

            save_corrected_image(&img, &corrected_path)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Error saving image: {}", e)))?;

            // Generate thumbnail
            let thumb_filename = format!("thumb_{}.png", stem);
            let thumb_path = image_dir.join(thumb_filename);

            generate_thumbnail(&corrected_path, &thumb_path, 500, 500, 6)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Error generating thumbnail: {}", e)))?;

            Ok((
                corrected_path.to_string_lossy().to_string(),
                thumb_path.to_string_lossy().to_string(),
            ))
        }
        Err(e) => {
            // Fallback: try to copy original file anyway
            error!("Warning: Could not open image with correction: {}. Trying direct copy...", e);

            let new_path = image_dir.join(filename);
            match fs::copy(original_path, &new_path) {
                Ok(_) => {
                    let filename_str = filename.to_string_lossy();
                    let stem = filename_str.split('.').next().unwrap_or(&filename_str);
                    let thumb_filename = format!("thumb_{}.png", stem);
                    let thumb_path = image_dir.join(thumb_filename);

                    // Try to generate thumbnail from original file
                    match generate_thumbnail(&new_path, &thumb_path, 500, 500, 6) {
                        Ok(_) => Ok((
                            new_path.to_string_lossy().to_string(),
                            thumb_path.to_string_lossy().to_string(),
                        )),
                        Err(thumb_err) => Err(io::Error::new(
                            io::ErrorKind::Other,
                            format!("Error generating thumbnail: {}", thumb_err)
                        ))
                    }
                }
                Err(copy_err) => Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Could not process image: {} | Copy error: {}", e, copy_err)
                ))
            }
        }
    }
}

pub fn delete_image(id: i64) -> io::Result<()> {
    let images_dir = Path::new("images");
    let image_dir = images_dir.join(id.to_string());
    fs::remove_dir_all(&image_dir)
}

pub fn open_in_file_explorer(path: &Path) -> io::Result<()> {
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Path does not exist",
        ));
    }
    if cfg!(target_os = "windows") {
        Command::new("explorer").arg(path).spawn()?;
    } else if cfg!(target_os = "linux") {
        Command::new("xdg-open").arg(path).spawn()?;
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(path).spawn()?;
    } else {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Unsupported OS",
        ));
    }

    Ok(())
}