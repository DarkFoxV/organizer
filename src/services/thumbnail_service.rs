use image::codecs::png::PngEncoder;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageReader, ColorType, ImageEncoder};
use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::time::Instant;
use log::{info, debug};

pub fn generate_thumbnail<P: AsRef<Path>>(
    input_path: P,
    output_path: P,
    max_width: u32,
    max_height: u32,
    compression_level: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    let img = ImageReader::open(&input_path)?;

    let img = img.decode()?;

    // Resize maintaining aspect ratio
    let resized = resize_preserving_aspect_ratio_optimized(&img, max_width, max_height);

    // Save as PNG
    save_image_as_png(&resized, &output_path, compression_level)?;

    let elapsed = start_time.elapsed();
    info!("Elapsed time: {:.3} seconds", elapsed.as_secs_f64());

    Ok(())
}

pub fn save_corrected_image<P: AsRef<Path>>(
    img: &DynamicImage,
    output_path: P,
) -> Result<(), Box<dyn std::error::Error>> {
    save_image_as_png(img, output_path, 6) // Uses balanced compression
}

fn save_image_as_png<P: AsRef<Path>>(
    img: &DynamicImage,
    output_path: P,
    compression_level: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    // Encode to PNG in buffer
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

    // Determine color type based on image
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

    // Second attempt: read as raw bytes and try different formats
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

fn fix_and_decode_image(bytes: &[u8]) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    // Try to fix JPEG header
    if bytes.len() > 10 && is_likely_jpeg(bytes) {
        let mut fixed_bytes = bytes.to_vec();

        // Check if it has correct SOI (Start of Image) marker
        if fixed_bytes[0] != 0xFF || fixed_bytes[1] != 0xD8 {
            // Look for the real start of JPEG
            for i in 0..fixed_bytes.len().saturating_sub(2) {
                if fixed_bytes[i] == 0xFF && fixed_bytes[i + 1] == 0xD8 {
                    fixed_bytes = fixed_bytes[i..].to_vec();
                    break;
                }
            }
        }

        // Try to decode corrected JPEG
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
            // Look for correct PNG signature
            for i in 0..fixed_bytes.len().saturating_sub(8) {
                if &fixed_bytes[i..i+8] == png_signature {
                    fixed_bytes = fixed_bytes[i..].to_vec();
                    break;
                }
            }
        }

        // Try to decode corrected PNG
        if let Ok(img) = image::load_from_memory_with_format(&fixed_bytes, image::ImageFormat::Png) {
            return Ok(img);
        }
    }

    Err("Could not fix image header".into())
}

fn is_likely_jpeg(bytes: &[u8]) -> bool {
    // Check if it contains common JPEG markers
    bytes.windows(2).any(|w| w == [0xFF, 0xD8]) || // SOI
        bytes.windows(4).any(|w| w == [0xFF, 0xE0, 0x00, 0x10]) || // APP0
        bytes.windows(4).any(|w| w == [0xFF, 0xE1, 0x00, 0x16]) // APP1
}

fn is_likely_png(bytes: &[u8]) -> bool {
    // Check if it contains PNG signature anywhere
    let png_signature = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    bytes.windows(8).any(|w| w == png_signature)
}

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