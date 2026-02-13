/// Property-based test for format support (Task 4.7)
///
/// **Property 5: Format Support**
/// **Validates: Requirements 4.5, 4.6**
///
/// For any valid GLTF/GLB mesh file or PNG/JPG/HDR texture file,
/// the asset server should successfully load and process it into
/// the corresponding runtime format (Mesh or Texture).
use luminara_render::{Mesh, Texture, TextureFormat};
use proptest::prelude::*;
use std::io::Cursor;

// ============================================================================
// GLTF/GLB Format Support Tests
// ============================================================================

/// Strategy to generate valid GLB triangle meshes with random vertex positions
fn arb_glb_triangle() -> impl Strategy<Value = Vec<u8>> {
    // Generate 3 random vertices for a triangle
    prop::array::uniform3(prop::array::uniform3(-10.0f32..10.0f32))
        .prop_map(|positions| create_glb_with_positions(positions))
}

/// Creates a valid GLB file with specified vertex positions
fn create_glb_with_positions(positions: [[f32; 3]; 3]) -> Vec<u8> {
    let json = r#"{
        "asset": {"version": "2.0"},
        "scene": 0,
        "scenes": [{"nodes": [0]}],
        "nodes": [{"mesh": 0}],
        "meshes": [{
            "primitives": [{
                "attributes": {
                    "POSITION": 0,
                    "NORMAL": 1
                },
                "indices": 2
            }]
        }],
        "accessors": [
            {
                "bufferView": 0,
                "componentType": 5126,
                "count": 3,
                "type": "VEC3",
                "min": [-10.0, -10.0, -10.0],
                "max": [10.0, 10.0, 10.0]
            },
            {
                "bufferView": 1,
                "componentType": 5126,
                "count": 3,
                "type": "VEC3"
            },
            {
                "bufferView": 2,
                "componentType": 5123,
                "count": 3,
                "type": "SCALAR"
            }
        ],
        "bufferViews": [
            {"buffer": 0, "byteOffset": 0, "byteLength": 36},
            {"buffer": 0, "byteOffset": 36, "byteLength": 36},
            {"buffer": 0, "byteOffset": 72, "byteLength": 6}
        ],
        "buffers": [{"byteLength": 78}]
    }"#;

    // Binary data: positions (3 vec3), normals (3 vec3), indices (3 u16)
    let mut bin_data = Vec::new();

    // Positions (3 vertices * 3 floats * 4 bytes = 36 bytes)
    for pos in &positions {
        bin_data.extend_from_slice(&pos[0].to_le_bytes());
        bin_data.extend_from_slice(&pos[1].to_le_bytes());
        bin_data.extend_from_slice(&pos[2].to_le_bytes());
    }

    // Normals (3 vertices * 3 floats * 4 bytes = 36 bytes)
    // Use a simple upward normal for all vertices
    for _ in 0..3 {
        bin_data.extend_from_slice(&0.0f32.to_le_bytes()); // nx
        bin_data.extend_from_slice(&0.0f32.to_le_bytes()); // ny
        bin_data.extend_from_slice(&1.0f32.to_le_bytes()); // nz
    }

    // Indices (3 u16 * 2 bytes = 6 bytes)
    bin_data.extend_from_slice(&0u16.to_le_bytes());
    bin_data.extend_from_slice(&1u16.to_le_bytes());
    bin_data.extend_from_slice(&2u16.to_le_bytes());

    // Build GLB
    let json_bytes = json.as_bytes();
    let json_padding = (4 - (json_bytes.len() % 4)) % 4;
    let json_length = json_bytes.len() + json_padding;
    let total_length = 12 + 8 + json_length + 8 + bin_data.len();

    let mut glb = Vec::new();

    // GLB header
    glb.extend_from_slice(b"glTF");
    glb.extend_from_slice(&2u32.to_le_bytes());
    glb.extend_from_slice(&(total_length as u32).to_le_bytes());

    // JSON chunk
    glb.extend_from_slice(&(json_length as u32).to_le_bytes());
    glb.extend_from_slice(b"JSON");
    glb.extend_from_slice(json_bytes);
    for _ in 0..json_padding {
        glb.push(b' ');
    }

    // BIN chunk
    glb.extend_from_slice(&(bin_data.len() as u32).to_le_bytes());
    glb.extend_from_slice(b"BIN\0");
    glb.extend_from_slice(&bin_data);

    glb
}

// ============================================================================
// PNG Format Support Tests
// ============================================================================

/// Strategy to generate valid PNG images with random dimensions and colors
fn arb_png_image() -> impl Strategy<Value = Vec<u8>> {
    (2u32..16, 2u32..16, prop::collection::vec(any::<u8>(), 4)).prop_map(
        |(width, height, color)| {
            create_png_image(width, height, [color[0], color[1], color[2], color[3]])
        },
    )
}

/// Creates a valid PNG image with specified dimensions and solid color
fn create_png_image(width: u32, height: u32, color: [u8; 4]) -> Vec<u8> {
    let mut img_buffer: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> =
        image::ImageBuffer::new(width, height);

    // Fill with solid color
    for pixel in img_buffer.pixels_mut() {
        *pixel = image::Rgba(color);
    }

    // Encode to PNG
    let mut png_data = Vec::new();
    let mut cursor = Cursor::new(&mut png_data);
    img_buffer
        .write_to(&mut cursor, image::ImageFormat::Png)
        .expect("Failed to encode PNG");

    png_data
}

// ============================================================================
// JPEG Format Support Tests
// ============================================================================

/// Strategy to generate valid JPEG images with random dimensions and colors
fn arb_jpeg_image() -> impl Strategy<Value = Vec<u8>> {
    (2u32..16, 2u32..16, prop::collection::vec(any::<u8>(), 3)).prop_map(
        |(width, height, color)| create_jpeg_image(width, height, [color[0], color[1], color[2]]),
    )
}

/// Creates a valid JPEG image with specified dimensions and solid color
fn create_jpeg_image(width: u32, height: u32, color: [u8; 3]) -> Vec<u8> {
    let mut img_buffer: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> =
        image::ImageBuffer::new(width, height);

    // Fill with solid color
    for pixel in img_buffer.pixels_mut() {
        *pixel = image::Rgb(color);
    }

    // Encode to JPEG
    let mut jpeg_data = Vec::new();
    let mut cursor = Cursor::new(&mut jpeg_data);
    img_buffer
        .write_to(&mut cursor, image::ImageFormat::Jpeg)
        .expect("Failed to encode JPEG");

    jpeg_data
}

// ============================================================================
// HDR Format Support Tests
// ============================================================================

/// Strategy to generate valid HDR images with random dimensions
fn arb_hdr_image() -> impl Strategy<Value = Vec<u8>> {
    (2u32..8, 2u32..8, prop::array::uniform3(0.0f32..10.0f32))
        .prop_map(|(width, height, color)| create_hdr_image(width, height, color))
}

/// Creates a valid HDR (Radiance RGBE) image with specified dimensions and color
fn create_hdr_image(width: u32, height: u32, color: [f32; 3]) -> Vec<u8> {
    use image::codecs::hdr::HdrEncoder;

    // Create HDR pixel data
    let pixel_count = (width * height) as usize;
    let mut pixels = Vec::with_capacity(pixel_count);

    for _ in 0..pixel_count {
        pixels.push(image::Rgb(color));
    }

    // Encode to HDR format
    let mut hdr_data = Vec::new();
    let mut cursor = Cursor::new(&mut hdr_data);
    let encoder = HdrEncoder::new(&mut cursor);
    encoder
        .encode(&pixels, width as usize, height as usize)
        .expect("Failed to encode HDR");

    hdr_data
}

// ============================================================================
// Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: For any valid GLB file, Mesh::from_gltf should successfully load it
    #[test]
    fn prop_glb_format_support(glb_data in arb_glb_triangle()) {
        let result = Mesh::from_gltf(&glb_data);

        prop_assert!(result.is_ok(), "Failed to load valid GLB: {:?}", result.err());

        let meshes = result.unwrap();
        prop_assert!(!meshes.is_empty(), "GLB should contain at least one mesh");

        let mesh = &meshes[0];
        prop_assert_eq!(mesh.vertices.len(), 3, "Triangle should have 3 vertices");
        prop_assert_eq!(mesh.indices.len(), 3, "Triangle should have 3 indices");

        // Verify all vertex data is finite (no NaN or infinity)
        for vertex in &mesh.vertices {
            prop_assert!(vertex.position[0].is_finite(), "Vertex position X should be finite");
            prop_assert!(vertex.position[1].is_finite(), "Vertex position Y should be finite");
            prop_assert!(vertex.position[2].is_finite(), "Vertex position Z should be finite");
            prop_assert!(vertex.normal[0].is_finite(), "Vertex normal X should be finite");
            prop_assert!(vertex.normal[1].is_finite(), "Vertex normal Y should be finite");
            prop_assert!(vertex.normal[2].is_finite(), "Vertex normal Z should be finite");
        }

        // Verify AABB is computed correctly
        prop_assert!(mesh.aabb.min.x.is_finite(), "AABB min X should be finite");
        prop_assert!(mesh.aabb.max.x.is_finite(), "AABB max X should be finite");
        prop_assert!(mesh.aabb.min.x <= mesh.aabb.max.x, "AABB min X should be <= max X");
        prop_assert!(mesh.aabb.min.y <= mesh.aabb.max.y, "AABB min Y should be <= max Y");
        prop_assert!(mesh.aabb.min.z <= mesh.aabb.max.z, "AABB min Z should be <= max Z");
    }

    /// Property: For any valid PNG file, Texture::from_bytes should successfully load it
    #[test]
    fn prop_png_format_support(png_data in arb_png_image()) {
        let result = Texture::from_bytes(&png_data);

        prop_assert!(result.is_ok(), "Failed to load valid PNG: {:?}", result.err());

        let texture = result.unwrap();
        prop_assert!(texture.data.width >= 2, "PNG width should be at least 2");
        prop_assert!(texture.data.height >= 2, "PNG height should be at least 2");
        prop_assert_eq!(texture.data.format, TextureFormat::Rgba8, "PNG should load as Rgba8");

        // Verify data size matches dimensions
        let expected_size = (texture.data.width * texture.data.height * 4) as usize;
        prop_assert_eq!(texture.data.data.len(), expected_size,
            "PNG data size should match width * height * 4");
    }

    /// Property: For any valid JPEG file, Texture::from_bytes should successfully load it
    #[test]
    fn prop_jpeg_format_support(jpeg_data in arb_jpeg_image()) {
        let result = Texture::from_bytes(&jpeg_data);

        prop_assert!(result.is_ok(), "Failed to load valid JPEG: {:?}", result.err());

        let texture = result.unwrap();
        prop_assert!(texture.data.width >= 2, "JPEG width should be at least 2");
        prop_assert!(texture.data.height >= 2, "JPEG height should be at least 2");
        prop_assert_eq!(texture.data.format, TextureFormat::Rgba8, "JPEG should load as Rgba8");

        // Verify data size matches dimensions
        let expected_size = (texture.data.width * texture.data.height * 4) as usize;
        prop_assert_eq!(texture.data.data.len(), expected_size,
            "JPEG data size should match width * height * 4");
    }

    /// Property: For any valid HDR file, Texture::from_bytes should successfully load it
    #[test]
    fn prop_hdr_format_support(hdr_data in arb_hdr_image()) {
        let result = Texture::from_bytes(&hdr_data);

        prop_assert!(result.is_ok(), "Failed to load valid HDR: {:?}", result.err());

        let texture = result.unwrap();
        prop_assert!(texture.data.width >= 2, "HDR width should be at least 2");
        prop_assert!(texture.data.height >= 2, "HDR height should be at least 2");
        prop_assert_eq!(texture.data.format, TextureFormat::Rgba32F, "HDR should load as Rgba32F");

        // Verify data size matches dimensions (4 floats * 4 bytes = 16 bytes per pixel)
        let expected_size = (texture.data.width * texture.data.height * 16) as usize;
        prop_assert_eq!(texture.data.data.len(), expected_size,
            "HDR data size should match width * height * 16");
    }

    /// Property: Different format files should load with correct format types
    #[test]
    fn prop_format_type_correctness(
        png_data in arb_png_image(),
        jpeg_data in arb_jpeg_image(),
        hdr_data in arb_hdr_image(),
    ) {
        // Load all three formats
        let png_texture = Texture::from_bytes(&png_data).unwrap();
        let jpeg_texture = Texture::from_bytes(&jpeg_data).unwrap();
        let hdr_texture = Texture::from_bytes(&hdr_data).unwrap();

        // Verify each has the correct format
        prop_assert_eq!(png_texture.data.format, TextureFormat::Rgba8,
            "PNG should be Rgba8");
        prop_assert_eq!(jpeg_texture.data.format, TextureFormat::Rgba8,
            "JPEG should be Rgba8");
        prop_assert_eq!(hdr_texture.data.format, TextureFormat::Rgba32F,
            "HDR should be Rgba32F");
    }

    /// Property: Loaded meshes should have valid indices within vertex range
    #[test]
    fn prop_mesh_indices_valid(glb_data in arb_glb_triangle()) {
        let meshes = Mesh::from_gltf(&glb_data).unwrap();
        let mesh = &meshes[0];

        let vertex_count = mesh.vertices.len() as u32;

        for &index in &mesh.indices {
            prop_assert!(index < vertex_count,
                "Index {} should be less than vertex count {}", index, vertex_count);
        }
    }

    /// Property: Loaded textures should have non-zero dimensions
    #[test]
    fn prop_texture_dimensions_nonzero(
        format in prop_oneof![
            arb_png_image(),
            arb_jpeg_image(),
            arb_hdr_image(),
        ]
    ) {
        let texture = Texture::from_bytes(&format).unwrap();

        prop_assert!(texture.data.width > 0, "Texture width should be > 0");
        prop_assert!(texture.data.height > 0, "Texture height should be > 0");
        prop_assert!(!texture.data.data.is_empty(), "Texture data should not be empty");
    }
}

// ============================================================================
// Unit Tests for Edge Cases
// ============================================================================

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_minimal_glb_loads() {
        let glb = create_glb_with_positions([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);

        let result = Mesh::from_gltf(&glb);
        assert!(result.is_ok(), "Minimal GLB should load successfully");
    }

    #[test]
    fn test_minimal_png_loads() {
        let png = create_png_image(2, 2, [255, 0, 0, 255]);

        let result = Texture::from_bytes(&png);
        assert!(result.is_ok(), "Minimal PNG should load successfully");
    }

    #[test]
    fn test_minimal_jpeg_loads() {
        let jpeg = create_jpeg_image(2, 2, [255, 0, 0]);

        let result = Texture::from_bytes(&jpeg);
        assert!(result.is_ok(), "Minimal JPEG should load successfully");
    }

    #[test]
    fn test_minimal_hdr_loads() {
        let hdr = create_hdr_image(2, 2, [1.0, 0.5, 0.25]);

        let result = Texture::from_bytes(&hdr);
        assert!(result.is_ok(), "Minimal HDR should load successfully");
    }

    #[test]
    fn test_invalid_format_fails() {
        let invalid_data = vec![0u8; 100];

        let result = Texture::from_bytes(&invalid_data);
        assert!(result.is_err(), "Invalid data should fail to load");
    }
}
