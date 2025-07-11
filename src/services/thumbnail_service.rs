use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageReader};
use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::time::Instant;

pub fn generate_thumbnail<P: AsRef<Path>>(
    input_path: P,
    output_path: P,
    max_width: u32,
    max_height: u32,
    quality: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    // Abrir e decodificar a imagem
    let img = ImageReader::open(&input_path)?.decode()?;

    // Redimensionar mantendo aspecto
    let resized = resize_preserving_aspect_ratio_optimized(&img, max_width, max_height);

    // Codificar para JPEG em buffer
    let mut bytes = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(Cursor::new(&mut bytes), quality);
    encoder.encode_image(&resized)?;

    // Salvar no disco
    fs::write(output_path, bytes)?;

    let elapsed = start_time.elapsed();
    println!("Tempo decorrido: {:.3} segundos", elapsed.as_secs_f64());

    Ok(())
}

// Reaproveita sua função de resize exata e cálculo de dimensões

fn resize_preserving_aspect_ratio_optimized(
    img: &DynamicImage,
    max_width: u32,
    max_height: u32,
) -> DynamicImage {
    let (width, height) = img.dimensions();

    if width <= max_width && height <= max_height {
        return img.clone();
    }

    let (new_width, new_height) = calculate_dimensions(width, height, max_width, max_height);

    let filter = if new_width <= 200 || new_height <= 200 {
        FilterType::Triangle
    } else {
        FilterType::Lanczos3
    };

    img.resize_exact(new_width, new_height, filter)
}

#[inline]
fn calculate_dimensions(width: u32, height: u32, max_width: u32, max_height: u32) -> (u32, u32) {
    let width_ratio = max_width as f32 / width as f32;
    let height_ratio = max_height as f32 / height as f32;
    let scale_ratio = width_ratio.min(height_ratio);

    (
        (width as f32 * scale_ratio).round() as u32,
        (height as f32 * scale_ratio).round() as u32,
    )
}