use luminara_render::{Texture, TextureFormat};

#[test]
fn test_load_png_texture() {
    // Create a minimal 2x2 PNG image (red, green, blue, white pixels)
    let png_data = create_test_png();
    
    let texture = Texture::from_bytes(&png_data).expect("Failed to load PNG");
    
    assert_eq!(texture.data.width, 2);
    assert_eq!(texture.data.height, 2);
    assert_eq!(texture.data.format, TextureFormat::Rgba8);
    assert_eq!(texture.data.data.len(), 16); // 2x2 pixels * 4 bytes
}

#[test]
fn test_load_invalid_data() {
    let invalid_data = vec![0u8; 100];
    
    let result = Texture::from_bytes(&invalid_data);
    assert!(result.is_err(), "Should fail to load invalid image data");
}

#[test]
fn test_texture_format_variants() {
    // Test Rgba8 format
    let texture = Texture::solid_color(1, 1, [255, 128, 64, 255]);
    assert_eq!(texture.data.format, TextureFormat::Rgba8);
    assert_eq!(texture.data.data.len(), 4);
}

/// Create a minimal valid PNG image (2x2 pixels)
fn create_test_png() -> Vec<u8> {
    use std::io::Cursor;
    
    // Create a simple 2x2 image with explicit u8 type
    let mut img_buffer: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = image::ImageBuffer::new(2, 2);
    
    // Set pixel colors: red, green, blue, white
    img_buffer.put_pixel(0, 0, image::Rgba([255, 0, 0, 255]));
    img_buffer.put_pixel(1, 0, image::Rgba([0, 255, 0, 255]));
    img_buffer.put_pixel(0, 1, image::Rgba([0, 0, 255, 255]));
    img_buffer.put_pixel(1, 1, image::Rgba([255, 255, 255, 255]));
    
    // Encode to PNG
    let mut png_data = Vec::new();
    let mut cursor = Cursor::new(&mut png_data);
    img_buffer.write_to(&mut cursor, image::ImageFormat::Png)
        .expect("Failed to encode PNG");
    
    png_data
}

#[test]
fn test_load_jpeg_texture() {
    // Create a minimal JPEG image
    let jpeg_data = create_test_jpeg();
    
    let texture = Texture::from_bytes(&jpeg_data).expect("Failed to load JPEG");
    
    assert_eq!(texture.data.width, 2);
    assert_eq!(texture.data.height, 2);
    assert_eq!(texture.data.format, TextureFormat::Rgba8);
}

/// Create a minimal valid JPEG image (2x2 pixels)
fn create_test_jpeg() -> Vec<u8> {
    use std::io::Cursor;
    
    // Create a simple 2x2 RGB image (JPEG doesn't support alpha)
    let mut img_buffer: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> = image::ImageBuffer::new(2, 2);
    
    // Set pixel colors
    img_buffer.put_pixel(0, 0, image::Rgb([255, 0, 0]));
    img_buffer.put_pixel(1, 0, image::Rgb([0, 255, 0]));
    img_buffer.put_pixel(0, 1, image::Rgb([0, 0, 255]));
    img_buffer.put_pixel(1, 1, image::Rgb([255, 255, 255]));
    
    // Encode to JPEG
    let mut jpeg_data = Vec::new();
    let mut cursor = Cursor::new(&mut jpeg_data);
    img_buffer.write_to(&mut cursor, image::ImageFormat::Jpeg)
        .expect("Failed to encode JPEG");
    
    jpeg_data
}

#[test]
fn test_texture_dimensions() {
    let texture = Texture::solid_color(16, 32, [128, 128, 128, 255]);
    
    assert_eq!(texture.data.width, 16);
    assert_eq!(texture.data.height, 32);
    assert_eq!(texture.data.data.len(), 16 * 32 * 4);
}
