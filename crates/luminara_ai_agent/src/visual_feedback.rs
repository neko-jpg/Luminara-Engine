// Requirements 11.1-11.7
// "VisualFeedbackSystem... screenshot capturer... visual annotations... viewport.capture MCP tool"

use luminara_core::world::World;
use luminara_math::Transform;
use std::path::PathBuf;

pub struct VisualFeedbackSystem {
    // capturer: ScreenshotCapturer,
}

#[derive(Default)]
pub struct CaptureConfig {
    pub resolution: (u32, u32),
    pub format: ImageFormat,
    pub annotations: AnnotationConfig,
}

#[derive(Default, PartialEq, Debug)]
pub enum ImageFormat {
    #[default]
    Jpeg,
    Png,
}

#[derive(Default)]
pub struct AnnotationConfig {
    pub show_entity_names: bool,
    pub show_bounding_boxes: bool,
    pub show_light_positions: bool,
    pub highlight_selected: bool,
}

impl Default for VisualFeedbackSystem {
    fn default() -> Self {
        Self {}
    }
}

impl VisualFeedbackSystem {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn capture_viewport(&self, world: &World, config: CaptureConfig) -> Vec<u8> {
        // Mock capture logic for MVP (headless/CLI environment)
        // We render a simple top-down view of the entities on the XZ plane.

        let (width, height) = config.resolution;
        let mut buffer = vec![0u8; (width * height * 3) as usize]; // RGB

        // Clear to dark blue background
        for i in (0..buffer.len()).step_by(3) {
            buffer[i] = 10; // R
            buffer[i + 1] = 10; // G
            buffer[i + 2] = 50; // B
        }

        // Camera settings for mock view
        let scale = 20.0; // World units visible width/height
        let offset_x = width as f32 / 2.0;
        let offset_y = height as f32 / 2.0;
        let ppu = width as f32 / scale; // Pixels per unit

        for entity in world.entities() {
            if let Some(transform) = world.get_component::<Transform>(entity) {
                let pos = transform.translation;
                // Project XZ plane to 2D image
                let px = (pos.x * ppu + offset_x) as i32;
                let py = (pos.z * ppu + offset_y) as i32;

                // Draw a small white dot (3x3 pixels) for each entity
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let nx = px + dx;
                        let ny = py + dy;
                        if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                            let idx = ((ny as u32 * width + nx as u32) * 3) as usize;
                            if idx + 2 < buffer.len() {
                                buffer[idx] = 255;
                                buffer[idx + 1] = 255;
                                buffer[idx + 2] = 255;
                            }
                        }
                    }
                }
            }
        }

        buffer
    }

    pub fn create_annotated_view(
        &self,
        screenshot: &[u8],
        world: &World,
        config: AnnotationConfig,
    ) -> Vec<u8> {
        // Start with the original screenshot
        let mut annotated = screenshot.to_vec();

        let len = annotated.len();
        if len == 0 {
            return annotated;
        }

        let pixel_count = len / 3;
        let width = (pixel_count as f64).sqrt() as usize;
        let height = width; // Assume square for simplicity in mock

        if width == 0 {
            return annotated;
        }

        let scale = 20.0;
        let offset_x = width as f32 / 2.0;
        let offset_y = height as f32 / 2.0;
        let ppu = width as f32 / scale;

        if config.show_bounding_boxes {
            for entity in world.entities() {
                if let Some(transform) = world.get_component::<Transform>(entity) {
                    let pos = transform.translation;
                    let px = (pos.x * ppu + offset_x) as i32;
                    let py = (pos.z * ppu + offset_y) as i32;

                    // Draw a red box around entity (10x10 pixels)
                    let box_size: i32 = 5;
                    for dy in -box_size..=box_size {
                        for dx in -box_size..=box_size {
                            // Only draw border
                            if dx.abs() == box_size || dy.abs() == box_size {
                                let nx = px + dx;
                                let ny = py + dy;
                                if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                                    let idx = ((ny as u32 * width as u32 + nx as u32) * 3) as usize;
                                    if idx + 2 < annotated.len() {
                                        annotated[idx] = 255; // R
                                        annotated[idx + 1] = 0; // G
                                        annotated[idx + 2] = 0; // B
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        annotated
    }

    pub fn compare_before_after(&self, _before: &[u8], _after: &[u8]) -> ComparisonResult {
        // Calculate pixel diff
        // Mock: 10% diff
        ComparisonResult {
            diff_percentage: 10.0,
            visual_diff_image: vec![],
        }
    }

    pub fn generate_scene_description(&self, world: &World) -> String {
        let count = world.entities().len();
        format!(
            "Scene contains {} entities. Entities are located on the XZ plane.",
            count
        )
    }

    pub fn compress_image(&self, image_data: &[u8], _target_size_reduction: f32) -> Vec<u8> {
        // Mock compression: Just return as is for now.
        // Real implementation would use image crate to save as JPEG.
        image_data.to_vec()
    }
}

pub struct ComparisonResult {
    pub diff_percentage: f32,
    pub visual_diff_image: Vec<u8>,
}
