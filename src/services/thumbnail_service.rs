use image::codecs::png::PngEncoder;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageReader, ColorType, ImageEncoder};
use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::time::Instant;
use iced::advanced::image::Handle;
use log::{info, debug};

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
    let resized = resize_preserving_aspect_ratio(image, max_width, max_height);

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

    // Choose filter based on resulting image size
    let filter = if new_width <= 200 || new_height <= 200 {
        FilterType::Triangle
    } else {
        FilterType::Lanczos3
    };

    img.resize_exact(new_width, new_height, filter)
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
fn save_image_as_png<P: AsRef<Path>>(
    img: &DynamicImage,
    output_path: P,
    compression_level: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    // Encode to PNG into buffer
    let mut bytes = Vec::new();

    // Define compression type based on level
    let compression_type = match compression_level {
        0..=3 => image::codecs::png::CompressionType::Fast,
        4..=6 => image::codecs::png::CompressionType::Default,
        7..=9 => image::codecs::png::CompressionType::Best,
        _ => image::codecs::png::CompressionType::Default,
    };

    let encoder = PngEncoder::new_with_quality(
        Cursor::new(&mut bytes),
        compression_type,
        image::codecs::png::FilterType::Adaptive,
    );

    // Determine color type based on the image
    let color_type = match img.color() {
        ColorType::L8 => ColorType::L8,
        ColorType::La8 => ColorType::La8,
        ColorType::Rgb8 => ColorType::Rgb8,
        ColorType::Rgba8 => ColorType::Rgba8,
        ColorType::L16 => ColorType::L16,
        ColorType::La16 => ColorType::La16,
        ColorType::Rgb16 => ColorType::Rgb16,
        ColorType::Rgba16 => ColorType::Rgba16,
        _ => ColorType::Rgba8, // Fallback to RGBA8
    };

    encoder.write_image(
        img.as_bytes(),
        img.width(),
        img.height(),
        color_type.into(),
    )?;

    // Save to disk
    fs::write(output_path, bytes)?;

    Ok(())
}

// ===================================
//         IMAGE LOADING AND FIXING
// ===================================

/// Opens and tries to fix an image
pub fn open_and_fix_image<P: AsRef<Path>>(
    input_path: P,
) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    // First attempt: open normally
    match ImageReader::open(&input_path) {
        Ok(reader) => match reader.decode() {
            Ok(img) => return Ok(img),
            Err(e) => {
                debug!("Error decoding image: {}. Trying to fix...", e);
            }
        },
        Err(e) => {
            debug!("Error opening file: {}. Trying to fix...", e);
        }
    }

    // Second attempt: read raw bytes and try different formats
    let bytes = fs::read(&input_path)?;

    // Try different decoders
    let formats = [
        image::ImageFormat::Jpeg,
        image::ImageFormat::Png,
        image::ImageFormat::Gif,
        image::ImageFormat::WebP,
        image::ImageFormat::Tiff,
        image::ImageFormat::Bmp,
    ];

    for format in &formats {
        match image::load_from_memory_with_format(&bytes, *format) {
            Ok(img) => {
                debug!("Successfully decoded as {:?}", format);
                return Ok(img);
            }
            Err(_) => continue,
        }
    }

    // Third attempt: try to fix specific headers
    if let Ok(img) = fix_and_decode_image(&bytes) {
        debug!("Successfully fixed and decoded image");
        return Ok(img);
    }

    Err("Could not open or fix the image".into())
}

/// Tries to fix and decode an image from bytes
fn fix_and_decode_image(bytes: &[u8]) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    // Try to fix JPEG header
    if bytes.len() > 10 && is_likely_jpeg(bytes) {
        let mut fixed_bytes = bytes.to_vec();

        // Check if SOI (Start of Image) marker is correct
        if fixed_bytes[0] != 0xFF || fixed_bytes[1] != 0xD8 {
            // Search for the actual JPEG start
            for i in 0..fixed_bytes.len().saturating_sub(2) {
                if fixed_bytes[i] == 0xFF && fixed_bytes[i + 1] == 0xD8 {
                    fixed_bytes = fixed_bytes[i..].to_vec();
                    break;
                }
            }
        }

        // Try to decode the fixed JPEG
        if let Ok(img) = image::load_from_memory_with_format(&fixed_bytes, image::ImageFormat::Jpeg) {
            return Ok(img);
        }
    }

    // Try to fix PNG header
    if bytes.len() > 8 && is_likely_png(bytes) {
        let mut fixed_bytes = bytes.to_vec();

        // Check PNG signature
        let png_signature = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        if &fixed_bytes[0..8] != png_signature {
            // Search for the correct PNG signature
            for i in 0..fixed_bytes.len().saturating_sub(8) {
                if &fixed_bytes[i..i+8] == png_signature {
                    fixed_bytes = fixed_bytes[i..].to_vec();
                    break;
                }
            }
        }

        // Try to decode the fixed PNG
        if let Ok(img) = image::load_from_memory_with_format(&fixed_bytes, image::ImageFormat::Png) {
            return Ok(img);
        }
    }

    Err("Could not fix image header".into())
}

/// Checks if the bytes are likely from a JPEG
fn is_likely_jpeg(bytes: &[u8]) -> bool {
    // Checks for common JPEG markers
    bytes.windows(2).any(|w| w == [0xFF, 0xD8]) || // SOI
        bytes.windows(4).any(|w| w == [0xFF, 0xE0, 0x00, 0x10]) || // APP0
        bytes.windows(4).any(|w| w == [0xFF, 0xE1, 0x00, 0x16]) // APP1
}

/// Checks if the bytes are likely from a PNG
fn is_likely_png(bytes: &[u8]) -> bool {
    // Checks if PNG signature is found somewhere
    let png_signature = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    bytes.windows(8).any(|w| w == png_signature)
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