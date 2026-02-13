use luminara_asset::AssetLoader;
use luminara_render::{Mesh, MeshLoader};
use std::path::Path;

#[test]
fn test_mesh_loader_with_valid_gltf() {
    // Create a minimal valid GLTF binary (GLB format)
    // This is a simple triangle mesh in GLB format
    let glb_data = create_minimal_glb_triangle();

    let loader = MeshLoader;
    let result = loader.load(&glb_data, Path::new("test.glb"));

    assert!(
        result.is_ok(),
        "Failed to load valid GLB: {:?}",
        result.err()
    );

    let mesh = result.unwrap();
    assert!(
        mesh.vertices.len() >= 3,
        "Mesh should have at least 3 vertices for a triangle"
    );
    assert!(
        mesh.indices.len() >= 3,
        "Mesh should have at least 3 indices for a triangle"
    );

    // Verify AABB is computed
    assert!(mesh.aabb.min.x.is_finite());
    assert!(mesh.aabb.max.x.is_finite());
}

#[test]
fn test_mesh_loader_with_invalid_gltf() {
    let loader = MeshLoader;
    let invalid_data = b"This is not a valid GLTF file";

    let result = loader.load(invalid_data, Path::new("invalid.glb"));
    assert!(result.is_err(), "Should fail to load invalid GLTF data");
}

#[test]
fn test_mesh_loader_extensions() {
    let loader = MeshLoader;
    let extensions = loader.extensions();

    assert_eq!(extensions.len(), 2);
    assert!(extensions.contains(&"gltf"));
    assert!(extensions.contains(&"glb"));
}

/// Creates a minimal valid GLB file containing a single triangle
fn create_minimal_glb_triangle() -> Vec<u8> {
    // GLB format:
    // Header (12 bytes): magic (4), version (4), length (4)
    // JSON chunk header (8 bytes): length (4), type (4)
    // JSON chunk data (padded to 4-byte boundary)
    // BIN chunk header (8 bytes): length (4), type (4)
    // BIN chunk data (padded to 4-byte boundary)

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
                "min": [-0.5, -0.5, 0.0],
                "max": [0.5, 0.5, 0.0]
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
    bin_data.extend_from_slice(&0.0f32.to_le_bytes()); // v0.x
    bin_data.extend_from_slice(&0.5f32.to_le_bytes()); // v0.y
    bin_data.extend_from_slice(&0.0f32.to_le_bytes()); // v0.z

    bin_data.extend_from_slice(&(-0.5f32).to_le_bytes()); // v1.x
    bin_data.extend_from_slice(&(-0.5f32).to_le_bytes()); // v1.y
    bin_data.extend_from_slice(&0.0f32.to_le_bytes()); // v1.z

    bin_data.extend_from_slice(&0.5f32.to_le_bytes()); // v2.x
    bin_data.extend_from_slice(&(-0.5f32).to_le_bytes()); // v2.y
    bin_data.extend_from_slice(&0.0f32.to_le_bytes()); // v2.z

    // Normals (3 vertices * 3 floats * 4 bytes = 36 bytes)
    for _ in 0..3 {
        bin_data.extend_from_slice(&0.0f32.to_le_bytes()); // nx
        bin_data.extend_from_slice(&0.0f32.to_le_bytes()); // ny
        bin_data.extend_from_slice(&1.0f32.to_le_bytes()); // nz
    }

    // Indices (3 u16 * 2 bytes = 6 bytes)
    bin_data.extend_from_slice(&0u16.to_le_bytes());
    bin_data.extend_from_slice(&1u16.to_le_bytes());
    bin_data.extend_from_slice(&2u16.to_le_bytes());

    // Pad JSON to 4-byte boundary
    let json_bytes = json.as_bytes();
    let json_padding = (4 - (json_bytes.len() % 4)) % 4;
    let json_length = json_bytes.len() + json_padding;

    // Calculate total length
    let total_length = 12 + 8 + json_length + 8 + bin_data.len();

    let mut glb = Vec::new();

    // GLB header
    glb.extend_from_slice(b"glTF"); // magic
    glb.extend_from_slice(&2u32.to_le_bytes()); // version
    glb.extend_from_slice(&(total_length as u32).to_le_bytes()); // length

    // JSON chunk header
    glb.extend_from_slice(&(json_length as u32).to_le_bytes()); // chunk length
    glb.extend_from_slice(b"JSON"); // chunk type

    // JSON chunk data
    glb.extend_from_slice(json_bytes);
    for _ in 0..json_padding {
        glb.push(b' '); // Pad with spaces
    }

    // BIN chunk header
    glb.extend_from_slice(&(bin_data.len() as u32).to_le_bytes()); // chunk length
    glb.extend_from_slice(b"BIN\0"); // chunk type

    // BIN chunk data
    glb.extend_from_slice(&bin_data);

    glb
}

#[test]
fn test_mesh_from_gltf_direct() {
    let glb_data = create_minimal_glb_triangle();
    let result = Mesh::from_gltf(&glb_data);

    assert!(result.is_ok(), "Failed to load GLB: {:?}", result.err());

    let meshes = result.unwrap();
    assert_eq!(meshes.len(), 1, "Should have exactly one mesh");

    let mesh = &meshes[0];
    assert_eq!(mesh.vertices.len(), 3, "Triangle should have 3 vertices");
    assert_eq!(mesh.indices.len(), 3, "Triangle should have 3 indices");

    // Verify vertex data
    for vertex in &mesh.vertices {
        assert!(vertex.position[0].is_finite());
        assert!(vertex.position[1].is_finite());
        assert!(vertex.position[2].is_finite());
        assert!(vertex.normal[0].is_finite());
        assert!(vertex.normal[1].is_finite());
        assert!(vertex.normal[2].is_finite());
    }

    // Verify AABB
    assert!(mesh.aabb.min.x <= mesh.aabb.max.x);
    assert!(mesh.aabb.min.y <= mesh.aabb.max.y);
    assert!(mesh.aabb.min.z <= mesh.aabb.max.z);
}
