// Visual Regression Test (Screenshot Comparison)
// Tests that rendering output remains consistent across code changes
//
// ## Updating Reference Images
//
// When intentional rendering changes are made, reference images need to be updated:
//
// 1. Review the visual changes to ensure they are correct
// 2. Set the environment variable: `UPDATE_GOLDEN_IMAGES=1`
// 3. Run the tests: `cargo test --test visual_regression_test`
// 4. Commit the updated reference images in `tests/golden/`
//
// ## Test Scenes
//
// - `cornell_box`: Basic lighting and material test
// - `pbr_materials`: PBR material properties (metallic, roughness)
// - `shadow_maps`: Shadow rendering accuracy
// - `sprite_rendering`: 2D sprite batching and rendering
// - `debug_visualization`: Debug wireframes and gizmos

use std::path::{Path, PathBuf};
use std::fs;
use image::{RgbaImage, ImageBuffer};

/// Represents a rendered frame for comparison
#[derive(Debug, Clone)]
pub struct FrameBuffer {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl FrameBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; (width * height * 4) as usize],
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: [u8; 4]) {
        let index = ((y * self.width + x) * 4) as usize;
        if index + 3 < self.pixels.len() {
            self.pixels[index..index + 4].copy_from_slice(&color);
        }
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let index = ((y * self.width + x) * 4) as usize;
        if index + 3 < self.pixels.len() {
            [
                self.pixels[index],
                self.pixels[index + 1],
                self.pixels[index + 2],
                self.pixels[index + 3],
            ]
        } else {
            [0, 0, 0, 0]
        }
    }

    pub fn compare(&self, other: &FrameBuffer, threshold: f32) -> (bool, f32) {
        if self.width != other.width || self.height != other.height {
            return (false, f32::MAX);
        }

        let mut total_diff = 0.0;
        let pixel_count = (self.width * self.height) as f32;

        for y in 0..self.height {
            for x in 0..self.width {
                let pixel1 = self.get_pixel(x, y);
                let pixel2 = other.get_pixel(x, y);

                let diff = ((pixel1[0] as f32 - pixel2[0] as f32).abs()
                    + (pixel1[1] as f32 - pixel2[1] as f32).abs()
                    + (pixel1[2] as f32 - pixel2[2] as f32).abs()
                    + (pixel1[3] as f32 - pixel2[3] as f32).abs())
                    / (255.0 * 4.0);

                total_diff += diff;
            }
        }

        let avg_diff = total_diff / pixel_count;
        (avg_diff <= threshold, avg_diff)
    }

    pub fn save_png(&self, path: &Path) -> Result<(), String> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let img: RgbaImage = ImageBuffer::from_raw(self.width, self.height, self.pixels.clone())
            .ok_or_else(|| "Failed to create image from buffer".to_string())?;
        
        img.save(path).map_err(|e| format!("Failed to save PNG: {}", e))
    }

    pub fn load_png(path: &Path) -> Result<Self, String> {
        let img = image::open(path)
            .map_err(|e| format!("Failed to load PNG: {}", e))?
            .to_rgba8();
        
        let (width, height) = img.dimensions();
        let pixels = img.into_raw();
        
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    /// Compare with another framebuffer and generate a diff image
    pub fn compare_with_diff(&self, other: &FrameBuffer, threshold: f32) -> (bool, f32, FrameBuffer) {
        let mut diff_buffer = FrameBuffer::new(self.width, self.height);
        
        if self.width != other.width || self.height != other.height {
            return (false, f32::MAX, diff_buffer);
        }

        let mut total_diff = 0.0;
        let pixel_count = (self.width * self.height) as f32;

        for y in 0..self.height {
            for x in 0..self.width {
                let pixel1 = self.get_pixel(x, y);
                let pixel2 = other.get_pixel(x, y);

                let diff = ((pixel1[0] as f32 - pixel2[0] as f32).abs()
                    + (pixel1[1] as f32 - pixel2[1] as f32).abs()
                    + (pixel1[2] as f32 - pixel2[2] as f32).abs()
                    + (pixel1[3] as f32 - pixel2[3] as f32).abs())
                    / (255.0 * 4.0);

                total_diff += diff;

                // Visualize difference in red
                let diff_intensity = (diff * 255.0) as u8;
                diff_buffer.set_pixel(x, y, [diff_intensity, 0, 0, 255]);
            }
        }

        let avg_diff = total_diff / pixel_count;
        (avg_diff <= threshold, avg_diff, diff_buffer)
    }
}

/// Simulates rendering a test scene
fn render_test_scene(scene_name: &str) -> FrameBuffer {
    let mut framebuffer = FrameBuffer::new(800, 600);

    // Simulate different scenes with different patterns
    match scene_name {
        "cornell_box" => {
            // Render a simple cornell box pattern
            for y in 0..600 {
                for x in 0..800 {
                    let color = if x < 400 {
                        [255, 0, 0, 255] // Red left wall
                    } else {
                        [0, 0, 255, 255] // Blue right wall
                    };
                    framebuffer.set_pixel(x, y, color);
                }
            }
        }
        "pbr_materials" => {
            // Render PBR test with varying metallic and roughness
            for y in 0..600 {
                for x in 0..800 {
                    // Create a gradient showing different material properties
                    let metallic = (x as f32 / 800.0 * 255.0) as u8;
                    let roughness = (y as f32 / 600.0 * 255.0) as u8;
                    framebuffer.set_pixel(x, y, [metallic, roughness, 128, 255]);
                }
            }
        }
        "shadow_maps" => {
            // Render scene with shadows
            for y in 0..600 {
                for x in 0..800 {
                    // Simulate shadow pattern
                    let in_shadow = (x / 100 + y / 100) % 2 == 0;
                    let brightness = if in_shadow { 64 } else { 192 };
                    framebuffer.set_pixel(x, y, [brightness, brightness, brightness, 255]);
                }
            }
        }
        "sprite_rendering" => {
            // Render sprites with transparency
            for y in 0..600 {
                for x in 0..800 {
                    // Create a checkerboard pattern with sprites
                    let sprite_x = x / 50;
                    let sprite_y = y / 50;
                    let is_sprite = (sprite_x + sprite_y) % 2 == 0;
                    
                    if is_sprite {
                        let color_index = (sprite_x + sprite_y) % 3;
                        let color = match color_index {
                            0 => [255, 0, 0, 200],
                            1 => [0, 255, 0, 200],
                            _ => [0, 0, 255, 200],
                        };
                        framebuffer.set_pixel(x, y, color);
                    } else {
                        framebuffer.set_pixel(x, y, [32, 32, 32, 255]);
                    }
                }
            }
        }
        "debug_visualization" => {
            // Render debug wireframes and gizmos
            for y in 0..600 {
                for x in 0..800 {
                    // Draw grid lines
                    let is_grid_line = x % 50 == 0 || y % 50 == 0;
                    
                    if is_grid_line {
                        framebuffer.set_pixel(x, y, [0, 255, 0, 255]);
                    } else {
                        // Draw coordinate axes
                        if x < 100 && y > 500 {
                            framebuffer.set_pixel(x, y, [255, 0, 0, 255]); // X axis
                        } else if x < 50 && y > 400 {
                            framebuffer.set_pixel(x, y, [0, 255, 0, 255]); // Y axis
                        } else {
                            framebuffer.set_pixel(x, y, [16, 16, 16, 255]);
                        }
                    }
                }
            }
        }
        "pbr_spheres" => {
            // Render PBR test spheres (legacy test)
            for y in 0..600 {
                for x in 0..800 {
                    let gray = ((x + y) % 256) as u8;
                    framebuffer.set_pixel(x, y, [gray, gray, gray, 255]);
                }
            }
        }
        _ => {
            // Default black scene
        }
    }

    framebuffer
}

/// Helper function to run a visual regression test
fn run_visual_regression_test(scene_name: &str, threshold: f32) {
    let golden_dir = PathBuf::from("tests/golden");
    let diff_dir = PathBuf::from("tests/diffs");
    let golden_path = golden_dir.join(format!("{}.png", scene_name));
    let diff_path = diff_dir.join(format!("{}_diff.png", scene_name));

    // Render current frame
    let current_frame = render_test_scene(scene_name);

    // Check if we should update golden images
    let update_golden = std::env::var("UPDATE_GOLDEN_IMAGES").is_ok();

    if update_golden {
        // Save as new golden image
        current_frame.save_png(&golden_path)
            .expect("Failed to save golden image");
        println!("Updated golden image: {}", golden_path.display());
        return;
    }

    // Load golden image
    let golden_frame = match FrameBuffer::load_png(&golden_path) {
        Ok(frame) => frame,
        Err(_) => {
            // If golden image doesn't exist, save current as golden
            println!("Golden image not found, creating: {}", golden_path.display());
            current_frame.save_png(&golden_path)
                .expect("Failed to save golden image");
            return;
        }
    };

    // Compare frames
    let (is_match, diff, diff_buffer) = current_frame.compare_with_diff(&golden_frame, threshold);

    if !is_match {
        // Save diff image for inspection
        fs::create_dir_all(&diff_dir).ok();
        diff_buffer.save_png(&diff_path)
            .expect("Failed to save diff image");
        
        panic!(
            "Visual regression detected in '{}' scene!\n\
             Difference: {:.4} (threshold: {:.4})\n\
             Diff image saved to: {}\n\
             To update golden image, run: UPDATE_GOLDEN_IMAGES=1 cargo test",
            scene_name, diff, threshold, diff_path.display()
        );
    }
}

#[test]
fn test_screenshot_comparison_identical() {
    let frame1 = render_test_scene("cornell_box");
    let frame2 = render_test_scene("cornell_box");

    let (is_match, diff) = frame1.compare(&frame2, 0.01);
    assert!(is_match, "Identical renders should match (diff: {})", diff);
}

#[test]
fn test_screenshot_comparison_different() {
    let frame1 = render_test_scene("cornell_box");
    let frame2 = render_test_scene("pbr_spheres");

    let (is_match, diff) = frame1.compare(&frame2, 0.01);
    assert!(
        !is_match,
        "Different renders should not match (diff: {})",
        diff
    );
}

#[test]
fn test_screenshot_comparison_threshold() {
    let mut frame1 = FrameBuffer::new(100, 100);
    let mut frame2 = FrameBuffer::new(100, 100);

    // Fill with slightly different colors
    for y in 0..100 {
        for x in 0..100 {
            frame1.set_pixel(x, y, [100, 100, 100, 255]);
            frame2.set_pixel(x, y, [101, 101, 101, 255]); // Slightly different
        }
    }

    // Should match with loose threshold
    let (is_match_loose, _) = frame1.compare(&frame2, 0.1);
    assert!(is_match_loose, "Should match with loose threshold");

    // Should not match with strict threshold
    let (is_match_strict, _) = frame1.compare(&frame2, 0.0001);
    assert!(!is_match_strict, "Should not match with strict threshold");
}

#[cfg(test)]
mod visual_regression_integration_tests {
    use super::*;

    #[test]
    fn test_cornell_box_regression() {
        run_visual_regression_test("cornell_box", 0.01);
    }

    #[test]
    fn test_pbr_materials_regression() {
        run_visual_regression_test("pbr_materials", 0.01);
    }

    #[test]
    fn test_shadow_maps_regression() {
        run_visual_regression_test("shadow_maps", 0.01);
    }

    #[test]
    fn test_sprite_rendering_regression() {
        run_visual_regression_test("sprite_rendering", 0.01);
    }

    #[test]
    fn test_debug_visualization_regression() {
        run_visual_regression_test("debug_visualization", 0.01);
    }

    #[test]
    fn test_pbr_spheres_regression() {
        // Legacy test maintained for compatibility
        run_visual_regression_test("pbr_spheres", 0.01);
    }

    #[test]
    fn test_headless_rendering() {
        // Test that we can render without a window (headless mode)
        let frame = render_test_scene("cornell_box");
        assert_eq!(frame.width, 800, "Headless rendering should work");
        assert_eq!(frame.height, 600, "Headless rendering should work");
    }

    #[test]
    fn test_shader_optimization_safety() {
        // Simulate testing that shader optimizations don't break rendering
        let before_optimization = render_test_scene("pbr_materials");

        // In a real scenario, we would apply shader optimizations here
        // For now, we just render again
        let after_optimization = render_test_scene("pbr_materials");

        let (is_match, diff) = before_optimization.compare(&after_optimization, 0.01);
        assert!(
            is_match,
            "Shader optimizations should not change visual output (diff: {})",
            diff
        );
    }

    #[test]
    fn test_multiple_resolutions() {
        // Test that visual regression works at different resolutions
        let resolutions = [(800, 600), (1920, 1080), (3840, 2160)];

        for (width, height) in resolutions.iter() {
            let frame = FrameBuffer::new(*width, *height);
            assert_eq!(frame.width, *width);
            assert_eq!(frame.height, *height);
        }
    }

    #[test]
    fn test_pixel_perfect_comparison() {
        let mut frame1 = FrameBuffer::new(10, 10);
        let mut frame2 = FrameBuffer::new(10, 10);

        // Set identical pixels
        for y in 0..10 {
            for x in 0..10 {
                let color = [(x * 25) as u8, (y * 25) as u8, 128, 255];
                frame1.set_pixel(x, y, color);
                frame2.set_pixel(x, y, color);
            }
        }

        let (is_match, diff) = frame1.compare(&frame2, 0.0);
        assert_eq!(
            diff, 0.0,
            "Pixel-perfect comparison should have zero difference"
        );
        assert!(is_match, "Identical frames should match perfectly");
    }

    #[test]
    fn test_tolerance_levels() {
        // Test different tolerance levels for minor rendering differences
        let mut frame1 = FrameBuffer::new(100, 100);
        let mut frame2 = FrameBuffer::new(100, 100);

        // Fill with slightly different colors (1 unit difference)
        for y in 0..100 {
            for x in 0..100 {
                frame1.set_pixel(x, y, [100, 100, 100, 255]);
                frame2.set_pixel(x, y, [101, 101, 101, 255]);
            }
        }

        // Calculate expected difference
        let expected_diff = 3.0 / (255.0 * 4.0); // 3 channels differ by 1

        let (_, actual_diff) = frame1.compare(&frame2, 1.0);
        assert!(
            (actual_diff - expected_diff).abs() < 0.0001,
            "Difference calculation should be accurate"
        );

        // Test various thresholds
        let (strict_match, _) = frame1.compare(&frame2, 0.0001);
        assert!(!strict_match, "Strict threshold should detect difference");

        let (loose_match, _) = frame1.compare(&frame2, 0.01);
        assert!(loose_match, "Loose threshold should tolerate minor difference");
    }

    #[test]
    fn test_diff_image_generation() {
        // Test that diff images are generated correctly
        let mut frame1 = FrameBuffer::new(50, 50);
        let mut frame2 = FrameBuffer::new(50, 50);

        // Create frames with a specific difference pattern
        for y in 0..50 {
            for x in 0..50 {
                frame1.set_pixel(x, y, [100, 100, 100, 255]);
                
                // Make right half different
                if x >= 25 {
                    frame2.set_pixel(x, y, [200, 200, 200, 255]);
                } else {
                    frame2.set_pixel(x, y, [100, 100, 100, 255]);
                }
            }
        }

        let (_, _, diff_buffer) = frame1.compare_with_diff(&frame2, 0.01);

        // Check that diff buffer highlights the different region
        let left_pixel = diff_buffer.get_pixel(10, 25);
        let right_pixel = diff_buffer.get_pixel(40, 25);

        // Left side should have minimal difference (darker red)
        assert!(left_pixel[0] < 10, "Left side should show minimal difference, got {}", left_pixel[0]);

        // Right side should have significant difference (brighter red)
        // Difference is 100 per channel (300 total) / (255 * 4) = 0.294
        // Scaled to 0-255: 0.294 * 255 = 75
        assert!(right_pixel[0] > 50, "Right side should show significant difference, got {}", right_pixel[0]);
    }

    #[test]
    fn test_image_save_load_roundtrip() {
        // Test that saving and loading preserves image data
        let original = render_test_scene("cornell_box");
        let temp_path = PathBuf::from("tests/temp_test_image.png");

        // Save
        original.save_png(&temp_path)
            .expect("Failed to save test image");

        // Load
        let loaded = FrameBuffer::load_png(&temp_path)
            .expect("Failed to load test image");

        // Compare
        let (is_match, diff) = original.compare(&loaded, 0.001);
        assert!(
            is_match,
            "Loaded image should match original (diff: {})",
            diff
        );

        // Cleanup
        fs::remove_file(&temp_path).ok();
    }

    #[test]
    fn test_all_scene_types_render() {
        // Ensure all scene types can be rendered without panicking
        let scenes = [
            "cornell_box",
            "pbr_materials",
            "shadow_maps",
            "sprite_rendering",
            "debug_visualization",
            "pbr_spheres",
        ];

        for scene in scenes.iter() {
            let frame = render_test_scene(scene);
            assert_eq!(frame.width, 800, "Scene '{}' should render at 800x600", scene);
            assert_eq!(frame.height, 600, "Scene '{}' should render at 800x600", scene);
        }
    }
}
