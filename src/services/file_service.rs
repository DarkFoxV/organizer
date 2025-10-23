use crate::config::get_settings;
use crate::dtos::image_dto::ImageDTO;
use crate::services::image_processor::generate_thumbnail_from_image;
use crate::utils::get_exe_dir;
use image::DynamicImage;
use log::{debug, info, warn};
use natord::compare;
use std::fs::{self, DirEntry};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::models::enums::image_type::ImageType;

// ===================================
//         UTILITY FUNCTIONS
// ===================================

pub fn detect_image_format(bytes: &[u8]) -> image::ImageFormat {
    if let Some(kind) = infer::get(bytes) {
        match kind.mime_type() {
            "image/jpeg" => image::ImageFormat::Jpeg,
            "image/png" => image::ImageFormat::Png,
            "image/gif" => image::ImageFormat::Gif,
            "image/webp" => image::ImageFormat::WebP,
            "image/bmp" => image::ImageFormat::Bmp,
            "image/tiff" => image::ImageFormat::Tiff,
            _ => image::ImageFormat::Png,
        }
    } else {
        image::ImageFormat::Png
    }
}

fn format_to_extension(format: image::ImageFormat) -> &'static str {
    match format {
        image::ImageFormat::Jpeg => "jpg",
        image::ImageFormat::Png => "png",
        image::ImageFormat::Gif => "gif",
        image::ImageFormat::WebP => "webp",
        image::ImageFormat::Bmp => "bmp",
        image::ImageFormat::Tiff => "tiff",
        _ => "png",
    }
}

pub fn save_image_file_with_thumbnail(
    id: i64,
    image: DynamicImage,
    original_format: image::ImageFormat,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let image_dir = get_exe_dir().join("images").join(id.to_string());
    if !image_dir.exists() {
        fs::create_dir_all(&image_dir)?;
    }

    let extension = format_to_extension(original_format);
    let image_filename = format!("image_{}.{}", id, extension);
    let image_path = image_dir.join(&image_filename);
    let thumb_path = image_dir.join(format!("thumb_image_{}.png", id));

    // Salvar no formato original
    image.save(&image_path)?;

    // Thumbnail continua em PNG
    let thumb_compression = get_settings().config.thumb_compression.unwrap_or(9);
    generate_thumbnail_from_image(&image, &thumb_path, 500, 500, thumb_compression)?;

    Ok((
        image_path.to_string_lossy().to_string(),
        thumb_path.to_string_lossy().to_string(),
    ))
}

pub fn save_images_from_folder_with_thumbnails(
    id: i64,
    folder_path: &Path,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let base_dir = get_exe_dir();
    let image_dir = base_dir.join("images").join(id.to_string());

    if !image_dir.exists() {
        fs::create_dir_all(&image_dir)?;
    }

    let thumb_compression = get_settings().config.thumb_compression.unwrap_or(9);

    let mut entries: Vec<DirEntry> = fs::read_dir(folder_path)?
        .filter_map(Result::ok)
        .filter(|e| {
            let path = e.path();
            path.is_file() && is_image_file(&path)
        })
        .collect();

    entries.sort_by(|a, b| {
        let name_a = a.file_name().to_string_lossy().to_lowercase();
        let name_b = b.file_name().to_string_lossy().to_lowercase();
        name_a.cmp(&name_b)
    });

    let mut saved_paths = Vec::new();
    let mut index = 0;

    let folder_thumb_path = image_dir.join("thumb_folder.png");
    if let Some(first_entry) = entries.first() {
        let bytes = fs::read(first_entry.path())?;
        let first_image = image::load_from_memory(&bytes)?;
        generate_thumbnail_from_image(
            &first_image,
            &folder_thumb_path,
            500,
            500,
            thumb_compression,
        )?;
        info!("Created folder thumbnail: {}", folder_thumb_path.display());
    }

    for entry in entries {
        let path = entry.path();

        let bytes = fs::read(&path)?;
        let original_format = detect_image_format(&bytes);
        let image = image::load_from_memory(&bytes)?;

        let extension = format_to_extension(original_format);

        let image_filename = format!("image_{}_{}.{}", id, index, extension);
        let image_path = image_dir.join(&image_filename);
        let thumb_path = image_dir.join(format!("thumb_image_{}_{}.png", id, index));

        image.save(&image_path)?;

        generate_thumbnail_from_image(&image, &thumb_path, 500, 500, thumb_compression)?;

        saved_paths.push((
            image_dir.to_string_lossy().to_string(),
            thumb_path.to_string_lossy().to_string(),
        ));

        index += 1;
    }

    let json_path = image_dir.join("meta.json");
    let index_json = serde_json::json!({
        "image_count": index,
        "next_index": index,
        "folder_thumb": folder_thumb_path.to_string_lossy().to_string()
    });
    fs::write(json_path, serde_json::to_string_pretty(&index_json)?)?;

    Ok(saved_paths)
}

// ===================================
//         DELETION FUNCTIONS
// ===================================

pub async fn delete_image(path: &str, image_type: ImageType) -> Result<(), io::Error> {
    let image_path = Path::new(path);
    info!("Deleting {:?} at {}", image_type, image_path.display());

    if !image_path.exists() {
        warn!("Path does not exist: {}", image_path.display());
        return Err(io::Error::new(io::ErrorKind::NotFound, "Path does not exist"));
    }

    match image_type {
        ImageType::FromFolder => {
            delete_single_file_with_thumbnail(path).await?;

            if let Some(parent) = image_path.parent() {
                if count_image_files_in_folder(parent)? == 0 {
                    delete_entire_folder(parent).await?;
                }
            }
            Ok(())
        }
        ImageType::Image => {
            delete_single_file_with_thumbnail(path).await?;

            if let Some(parent) = image_path.parent() {
                delete_entire_folder(parent).await?;
            }
            Ok(())
        }
        ImageType::Folder => delete_entire_folder(image_path).await,
    }
}

async fn delete_single_file_with_thumbnail(path: &str) -> Result<(), io::Error> {
    let image_path = Path::new(path);
    if image_path.exists() {
        fs::remove_file(image_path)?;
        info!("Deleted file: {}", image_path.display());

        if let Some(parent) = image_path.parent() {
            if let Some(name) = image_path.file_name().and_then(|n| n.to_str()) {
                let thumb_name = if name.starts_with("image_") {
                    format!("thumb_{}.png", name.split('.').next().unwrap())
                } else {
                    format!("thumb_{}", name)
                };
                let thumb_path = parent.join(thumb_name);
                if thumb_path.exists() {
                    fs::remove_file(&thumb_path)?;
                    info!("Deleted thumbnail: {}", thumb_path.display());
                }
            }
        }
    } else {
        debug!("File does not exist: {}", image_path.display());
    }
    Ok(())
}

async fn delete_entire_folder(folder_path: &Path) -> Result<(), io::Error> {
    if !folder_path.exists() {
        warn!("Folder does not exist: {}", folder_path.display());
        return Ok(());
    }
    if folder_path.file_name().and_then(|n| n.to_str()) == Some("images") {
        return Err(io::Error::new(io::ErrorKind::PermissionDenied, "Cannot delete root images folder"));
    }
    fs::remove_dir_all(folder_path)?;
    info!("Deleted folder: {}", folder_path.display());
    Ok(())
}


// ===================================
//         OTHER UTILITY FUNCTIONS
// ===================================

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

fn is_image_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        matches!(
            ext.to_lowercase().as_str(),
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "tiff" | "webp"
        )
    } else {
        false
    }
}

pub fn expand_folder_dto(image_dto: &ImageDTO) -> Vec<ImageDTO> {
    let folder_path = Path::new(&image_dto.path);
    if !folder_path.is_dir() {
        return vec![];
    }

    let entries = match fs::read_dir(folder_path) {
        Ok(e) => e,
        Err(_) => return vec![],
    };

    let mut files: Vec<(String, PathBuf)> = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_file() {
                if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {

                    if is_image_file(&path) && !filename.starts_with("thumb_") {
                        return Some((filename.to_string(), path));
                    }
                }
            }
            None
        })
        .collect();


    files.sort_by(|a, b| compare(&a.0, &b.0));

    let mut dtos = Vec::new();
    for (index, (filename, path)) in files.into_iter().enumerate() {

        let base_name = filename.split('.').next().unwrap_or(&filename);
        let thumb_path = folder_path.join(format!("thumb_{}.png", base_name));

        let dto = ImageDTO {
            id: index as i64,
            path: path.to_string_lossy().to_string(),
            thumbnail_path: thumb_path.to_string_lossy().to_string(),
            description: image_dto.description.clone(),
            tags: image_dto.tags.clone(),
            created_at: image_dto.created_at.clone(),
            is_folder: false,
            is_prepared: true,
        };

        dtos.push(dto);
    }
    dtos
}

fn count_image_files_in_folder(folder_path: &Path) -> Result<usize, io::Error> {
    if !folder_path.exists() || !folder_path.is_dir() {
        return Ok(0);
    }

    let mut count = 0;

    for entry in fs::read_dir(folder_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {

                if is_image_file(&path)
                    && !file_name.starts_with("thumb_")
                    && file_name != "meta.json"
                {
                    count += 1;
                }
            }
        }
    }

    Ok(count)
}
