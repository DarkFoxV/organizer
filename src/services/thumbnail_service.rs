use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageReader, ColorType};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::time::Instant;
use iced::advanced::image::Handle;
use log::info;
use fast_image_resize as fr;
use fast_image_resize::images::Image;

// ===================================
//         THUMBNAIL GENERATION
// ===================================

/// Generates a thumbnail from a specific image
pub fn generate_thumbnail_from_image<P: AsRef<Path>>(
    image: &DynamicImage,
    output_path: P,
    max_width: u32,
    max_height: u32,
    compression_level: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    // Resize while maintaining aspect ratio
    let resized = resize_with_fast_lib(image, max_width, max_height)?;

    // Save as PNG
    save_image_as_png(&resized, &output_path, compression_level)?;

    let elapsed = start_time.elapsed();
    info!("Thumbnail generated in {:.3} seconds", elapsed.as_secs_f64());

    Ok(())
}

// ===================================
//         IMAGE PROCESSING
// ===================================

/// Resizes an image while preserving the aspect ratio
fn resize_preserving_aspect_ratio(
    img: &DynamicImage,
    max_width: u32,
    max_height: u32,
) -> DynamicImage {
    let (width, height) = img.dimensions();

    // If the image is already within the limits, return a copy
    if width <= max_width && height <= max_height {
        return img.clone();
    }

    let (new_width, new_height) = calculate_dimensions(width, height, max_width, max_height);

    let filter = FilterType::Gaussian;

    img.resize_exact(new_width, new_height, filter)
}

fn resize_with_fast_lib(
    image: &DynamicImage,
    max_width: u32,
    max_height: u32,
) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    let (new_width, new_height) = calculate_dimensions(
        image.width(),
        image.height(),
        max_width,
        max_height,
    );

    // Converter para RGBA primeiro
    let mut rgba_image = image.to_rgba8();
    let (orig_width, orig_height) = rgba_image.dimensions();

    // Criar imagem de origem a partir do RGBA
    let src_image = Image::from_slice_u8(
        orig_width,
        orig_height,
        &mut rgba_image,
        fr::PixelType::U8x4,
    )?;

    // Criar imagem de destino
    let mut dst_image = Image::new(
        new_width,
        new_height,
        fr::PixelType::U8x4,
    );

    // Fazer resize
    let mut resizer = fr::Resizer::new();
    resizer.resize(&src_image, &mut dst_image, None)?;

    // Converter de volta para DynamicImage
    let buffer = dst_image.into_vec();
    let rgba_result = image::RgbaImage::from_raw(new_width, new_height, buffer)
        .ok_or("Failed to create RgbaImage")?;

    Ok(DynamicImage::ImageRgba8(rgba_result))
}

/// Calculates new dimensions while preserving aspect ratio
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

// ===================================
//         IMAGE SAVING
// ===================================

/// Saves an image as PNG with configurable compression
pub fn save_image_as_png<P: AsRef<Path>>(
    img: &DynamicImage,
    output_path: P,
    compression_level: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(output_path)?;
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, img.width(), img.height());

    // Set color type based on image
    match img.color() {
        ColorType::L8 => encoder.set_color(png::ColorType::Grayscale),
        ColorType::La8 => encoder.set_color(png::ColorType::GrayscaleAlpha),
        ColorType::Rgb8 => encoder.set_color(png::ColorType::Rgb),
        ColorType::Rgba8 => encoder.set_color(png::ColorType::Rgba),
        ColorType::L16 => encoder.set_color(png::ColorType::Grayscale),
        ColorType::La16 => encoder.set_color(png::ColorType::GrayscaleAlpha),
        ColorType::Rgb16 => encoder.set_color(png::ColorType::Rgb),
        ColorType::Rgba16 => encoder.set_color(png::ColorType::Rgba),
        _ => encoder.set_color(png::ColorType::Rgba),
    }

    encoder.set_depth(png::BitDepth::Eight);

    // Set compression level (0-9 → zlib levels)
    let level = match compression_level {
        0..=3 => png::Compression::Fast,
        4..=6 => png::Compression::Balanced,
        7..=9 => png::Compression::High,
        _ => png::Compression::Balanced,
    };
    encoder.set_compression(level);
    encoder.set_filter(png::Filter::Sub);

    let mut writer = encoder.write_header()?;
    writer.write_image_data(img.as_bytes())?;

    Ok(())
}

// ===================================
//         IMAGE LOADING
// ===================================

/// Opens an image file
pub fn open_image<P: AsRef<Path>>(
    input_path: P,
) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    let img = ImageReader::open(input_path)?.decode()?;
    Ok(img)
}

// ===================================
//         ICED INTEGRATION
// ===================================

/// Converts a DynamicImage to an Iced Handle
pub fn dynamic_image_to_rgba(dynamic_image: &DynamicImage) -> Handle {
    let rgba_image = dynamic_image.to_rgba8();
    let (width, height) = rgba_image.dimensions();
    let pixels = rgba_image.into_raw();
    Handle::from_rgba(width, height, pixels)
}