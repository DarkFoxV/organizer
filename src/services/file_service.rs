use crate::config::get_settings;
use crate::dtos::image_dto::ImageDTO;
use crate::services::image_processor;
use crate::services::image_processor::{generate_thumbnail_from_image, save_image_as_png};
use crate::utils::get_exe_dir;
use image::DynamicImage;
use log::{debug, info, warn};
use natord::compare;
use std::fs::{self, DirEntry};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

// ===================================
//         UTILITY FUNCTIONS
// ===================================

pub fn save_image_file_with_thumbnail(
    id: i64,
    image: DynamicImage,
    original_format: image::ImageFormat, // Adicionar este parâmetro
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let image_dir = get_exe_dir().join("images").join(id.to_string());
    if !image_dir.exists() {
        fs::create_dir_all(&image_dir)?;
    }

    // Determinar extensão baseada no formato
    let extension = match original_format {
        image::ImageFormat::Jpeg => "jpg",
        image::ImageFormat::Png => "png",
        image::ImageFormat::Gif => "gif",
        image::ImageFormat::WebP => "webp",
        image::ImageFormat::Bmp => "bmp",
        image::ImageFormat::Tiff => "tiff",
        _ => "png", // fallback
    };

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

    let image_compression = get_settings().config.image_compression.unwrap_or(5);
    let thumb_compression = get_settings().config.thumb_compression.unwrap_or(9);

    let mut entries: Vec<DirEntry> = fs::read_dir(folder_path)?
        .filter_map(Result::ok)
        .filter(|e| {
            let path = e.path();
            path.is_file() && is_image_file(&path)
        })
        .collect();

    // Ordenar por nome do arquivo (ordem alfabética)
    entries.sort_by(|a, b| {
        let name_a = a.file_name().to_string_lossy().to_lowercase();
        let name_b = b.file_name().to_string_lossy().to_lowercase();
        name_a.cmp(&name_b)
    });

    let mut saved_paths = Vec::new();
    let mut index = 0;

    // Criar thumb_folder usando o primeiro arquivo
    let folder_thumb_path = image_dir.join("thumb_folder.png");
    if let Some(first_entry) = entries.first() {
        let first_image = image_processor::open_image(&first_entry.path())?;
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
        let image = image_processor::open_image(&path)?;

        // Usar o padrão image_{id}_{incremento}
        let png_filename = format!("image_{}_{}.png", id, index);
        let image_path = image_dir.join(&png_filename);
        let thumb_path = image_dir.join(format!("thumb_image_{}_{}.png", id, index));

        save_image_as_png(&image, &image_path, image_compression)?;
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
//         SMART DELETION FUNCTIONS
// ===================================

/// Deleta uma imagem de forma inteligente baseado no contexto
/// - from_folder = true: deleta apenas o arquivo específico
/// - from_folder = false: deleta a pasta inteira se contém apenas um arquivo, senão deleta apenas o arquivo
pub async fn delete_image_smart(path: &str, from_folder: bool) -> Result<(), io::Error> {
    let image_path = Path::new(path);
    info!(
        "Smart deletion for path: {} (from_folder: {})",
        image_path.display(),
        from_folder
    );

    if !image_path.exists() {
        warn!("Path does not exist: {}", image_path.display());
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Path does not exist: {}", image_path.display()),
        ));
    }

    if from_folder {
        // Se é de uma pasta, delete apenas o arquivo específico
        delete_single_file_with_thumbnail(path).await
    } else {
        // Se não é de uma pasta, verifique se a pasta pai deve ser deletada completamente
        delete_image_or_folder_smart(path).await
    }
}

/// Deleta apenas um arquivo específico e seu thumbnail (usado para arquivos de pasta)
async fn delete_single_file_with_thumbnail(path: &str) -> Result<(), io::Error> {
    let image_path = Path::new(path);
    info!("Deleting single file: {}", image_path.display());

    if !image_path.exists() {
        warn!("File does not exist: {}", image_path.display());
        return Ok(()); // Não é erro se o arquivo já não existe
    }

    // Deletar o arquivo principal
    fs::remove_file(image_path)?;
    info!("Successfully deleted file: {}", image_path.display());

    // Deletar o thumbnail correspondente
    if let Some(parent_dir) = image_path.parent() {
        if let Some(file_name) = image_path.file_name() {
            let file_name_str = file_name.to_string_lossy();

            // Gerar o nome do thumbnail baseado no padrão
            let thumb_name = if file_name_str.starts_with("image_") {
                format!("thumb_{}", file_name_str)
            } else {
                format!("thumb_{}", file_name_str)
            };

            let thumb_path = parent_dir.join(thumb_name);

            if thumb_path.exists() {
                fs::remove_file(&thumb_path)?;
                info!("Successfully deleted thumbnail: {}", thumb_path.display());
            } else {
                debug!("Thumbnail does not exist: {}", thumb_path.display());
            }
        }
    }

    Ok(())
}

/// Analisa a pasta e decide se deve deletar apenas o arquivo ou a pasta inteira
async fn delete_image_or_folder_smart(path: &str) -> Result<(), io::Error> {
    let image_path = Path::new(path);

    let parent_dir = match image_path.parent() {
        Some(dir) => dir,
        None => {
            warn!(
                "Could not determine parent directory for: {}",
                image_path.display()
            );
            return delete_single_file_with_thumbnail(path).await;
        }
    };

    // Verificar se a pasta pai é uma pasta numerada dentro de "images/"
    let is_numbered_folder = parent_dir
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.chars().all(|c| c.is_ascii_digit()))
        .unwrap_or(false);

    if !is_numbered_folder {
        // Se não é uma pasta numerada, apenas delete o arquivo
        info!("Not a numbered folder, deleting single file");
        return delete_single_file_with_thumbnail(path).await;
    }

    // Contar quantos arquivos de imagem existem na pasta (excluindo thumbnails e meta.json)
    let image_count = count_image_files_in_folder(parent_dir)?;

    info!(
        "Found {} image files in folder: {}",
        image_count,
        parent_dir.display()
    );

    if image_count <= 1 {
        // Se há apenas um arquivo (ou nenhum), delete a pasta inteira
        info!(
            "Only one or no image files found, deleting entire directory: {}",
            parent_dir.display()
        );
        delete_entire_folder(parent_dir).await
    } else {
        // Se há múltiplos arquivos, delete apenas o arquivo específico
        info!("Multiple files found, deleting only the specific file");
        delete_single_file_with_thumbnail(path).await
    }
}

/// Conta quantos arquivos de imagem existem na pasta (excluindo thumbnails)
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
                // Contar apenas arquivos de imagem que não sejam thumbnails
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

/// Deleta uma pasta inteira com todo seu conteúdo
async fn delete_entire_folder(folder_path: &Path) -> Result<(), io::Error> {
    if !folder_path.exists() {
        warn!("Folder does not exist: {}", folder_path.display());
        return Ok(());
    }

    // Verificação de segurança: nunca deletar a pasta "images" raiz
    if let Some(folder_name) = folder_path.file_name().and_then(|n| n.to_str()) {
        if folder_name == "images" {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Cannot delete the root images folder",
            ));
        }
    }

    info!("Deleting entire directory: {}", folder_path.display());
    fs::remove_dir_all(folder_path)?;
    info!("Successfully deleted directory: {}", folder_path.display());

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
                    if filename.ends_with(".png") && !filename.starts_with("thumb_") {
                        return Some((filename.to_string(), path));
                    }
                }
            }
            None
        })
        .collect();

    // Ordenação natural (compreende números)
    files.sort_by(|a, b| compare(&a.0, &b.0));

    let mut dtos = Vec::new();
    for (index, (filename, path)) in files.into_iter().enumerate() {
        let thumb_path = folder_path.join(format!("thumb_{}", filename));

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
