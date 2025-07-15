use crate::services::thumbnail_service::generate_thumbnail_from_image;
use crate::utils::get_exe_dir;
use image::DynamicImage;
use log::{info, warn};
use std::fs;
use std::io;
use std::path::{Path};
use std::process::Command;

// ===================================
//         UTILITY FUNCTIONS
// ===================================

pub fn save_image_file_with_thumbnail(
    id: i64,
    image: DynamicImage,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    // Absolute path to the executable
    let base_dir = get_exe_dir();

    let images_dir = base_dir.join("images");
    if !images_dir.exists() {
        fs::create_dir_all(&images_dir)?;
    }

    let image_dir = images_dir.join(id.to_string());
    if !image_dir.exists() {
        fs::create_dir_all(&image_dir)?;
    }

    // Save
    let image_filename = format!("image_{}.png", id);
    let image_path = image_dir.join(&image_filename);
    image.save(&image_path)?;

    // Thumbnail
    let thumb_filename = format!("thumb_image_{}.png", id);
    let thumb_path = image_dir.join(&thumb_filename);
    generate_thumbnail_from_image(&image, &thumb_path, 500, 500, 6)?;

    Ok((
        image_path.to_string_lossy().to_string(),
        thumb_path.to_string_lossy().to_string(),
    ))
}

pub async fn delete_image_by_path(path: &str) -> Result<(), io::Error> {
    let image_path = Path::new(path);
    info!("Deleting image by path: {}", image_path.display());

    if let Some(parent_dir) = image_path.parent() {
        if parent_dir.exists() {
            info!("Deleting entire directory: {}", parent_dir.display());
            fs::remove_dir_all(parent_dir)?;
            info!("Successfully deleted directory: {}", parent_dir.display());
        } else {
            warn!("Parent directory does not exist: {}", parent_dir.display());
        }
    } else {
        warn!(
            "Could not determine parent directory for: {}",
            image_path.display()
        );
    }

    info!("File deletion completed for path: {}", path);
    Ok(())
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
        return Err(io::Error::new(io::ErrorKind::Other, "Unsupported OS"));
    }

    Ok(())
}

pub fn is_image_path(path: &str) -> bool {
    use std::path::Path;

    let path = Path::new(path);

    if !path.exists() {
        return false;
    }

    if !path.is_file() {
        return false;
    }

    if let Some(extension) = path.extension() {
        let ext = extension.to_string_lossy().to_lowercase();
        matches!(
            ext.as_str(),
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "ico" | "tiff" | "tif"
        )
    } else {
        false
    }
}
