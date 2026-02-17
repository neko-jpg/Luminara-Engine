// Visual Feedback System for AI Integration
// Requirements 29.1-29.7: Viewport capture, annotation overlay, multimodal LLM integration

use luminara_core::world::World;
use luminara_math::Transform;
use image::{ImageBuffer, Rgb, RgbImage, ImageFormat as ImgFormat};
use std::io::Cursor;
use std::time::Instant;

/// Visual Feedback System for capturing and annotating viewport images
pub struct VisualFeedbackSystem {
    /// Target resolution for captures
    target_resolution: (u32, u32),
    /// JPEG quality (0-100)
    jpeg_quality: u8,
}

/// Configuration for viewport capture
#[derive(Debug, Clone)]
pub struct CaptureConfig {
    pub resolution: (u32, u32),
    pub format: ImageFormat,
    pub annotations: AnnotationConfig,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            resolution: (512, 512),
            format: ImageFormat::Jpeg,
            annotations: AnnotationConfig::default(),
        }
    }
}

/// Image format for capture output
#[derive(Default, PartialEq, Debug, Clone, Copy)]
pub enum ImageFormat {
    #[default]
    Jpeg,
    Png,
}

/// Configuration for visual annotations
#[derive(Default, Debug, Clone)]
pub struct AnnotationConfig {
    pub show_entity_names: bool,
    pub show_bounding_boxes: bool,
    pub show_light_positions: bool,
    pub highlight_selected: bool,
}

/// Result of a viewport capture operation
pub struct CaptureResult {
    /// Encoded image data (JPEG or PNG)
    pub image_data: Vec<u8>,
    /// Image dimensions
    pub resolution: (u32, u32),
    /// File size in bytes
    pub file_size: usize,
    /// Capture latency in milliseconds
    pub capture_latency_ms: f32,
}

/// Result of before/after comparison
pub struct ComparisonResult {
    /// Percentage of pixels that differ
    pub diff_percentage: f32,
    /// Side-by-side comparison image
    pub comparison_image: Vec<u8>,
    /// Visual diff highlighting changes
    pub visual_diff_image: Vec<u8>,
}

impl Default for VisualFeedbackSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl VisualFeedbackSystem {
    /// Create a new Visual Feedback System with default settings
    pub fn new() -> Self {
        Self {
            target_resolution: (512, 512),
            jpeg_quality: 85, // Balance between quality and file size
        }
    }

    /// Create with custom resolution
    pub fn with_resolution(resolution: (u32, u32)) -> Self {
        Self {
            target_resolution: resolution,
            jpeg_quality: 85,
        }
    }

    /// Capture viewport with timing measurement
    /// Requirements 29.1: Generate 512x512 JPEG images with <100KB file size
    /// Requirements 29.7: Minimize capture latency (target: <50ms)
    pub fn capture_viewport(&self, world: &World, config: CaptureConfig) -> CaptureResult {
        let start = Instant::now();
        
        let (width, height) = config.resolution;
        
        // Render scene to RGB buffer
        let rgb_buffer = self.render_scene(world, width, height);
        
        // Apply annotations if requested
        let annotated_buffer = if config.annotations.show_bounding_boxes 
            || config.annotations.show_entity_names 
            || config.annotations.show_light_positions {
            self.apply_annotations(&rgb_buffer, world, width, height, &config.annotations)
        } else {
            rgb_buffer
        };
        
        // Encode to requested format
        let image_data = match config.format {
            ImageFormat::Jpeg => self.encode_jpeg(&annotated_buffer, width, height),
            ImageFormat::Png => self.encode_png(&annotated_buffer, width, height),
        };
        
        let latency = start.elapsed().as_secs_f32() * 1000.0;
        
        CaptureResult {
            file_size: image_data.len(),
            image_data,
            resolution: (width, height),
            capture_latency_ms: latency,
        }
    }

    /// Render scene to RGB buffer (mock implementation for headless environment)
    fn render_scene(&self, world: &World, width: u32, height: u32) -> Vec<u8> {
        let mut buffer = vec![0u8; (width * height * 3) as usize];
        
        // Clear to dark blue background
        for i in (0..buffer.len()).step_by(3) {
            buffer[i] = 10;      // R
            buffer[i + 1] = 10;  // G
            buffer[i + 2] = 50;  // B
        }
        
        // Camera settings for top-down view
        let scale = 20.0; // World units visible
        let offset_x = width as f32 / 2.0;
        let offset_y = height as f32 / 2.0;
        let ppu = width as f32 / scale; // Pixels per unit
        
        // Render entities as dots
        for entity in world.entities() {
            if let Some(transform) = world.get_component::<Transform>(entity) {
                let pos = transform.translation;
                let px = (pos.x * ppu + offset_x) as i32;
                let py = (pos.z * ppu + offset_y) as i32;
                
                // Draw 3x3 white dot for each entity
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

    /// Apply visual annotations to the rendered image
    /// Requirements 29.2: Overlay entity names, bounding boxes, light positions
    fn apply_annotations(
        &self,
        buffer: &[u8],
        world: &World,
        width: u32,
        height: u32,
        config: &AnnotationConfig,
    ) -> Vec<u8> {
        let mut annotated = buffer.to_vec();
        
        let scale = 20.0;
        let offset_x = width as f32 / 2.0;
        let offset_y = height as f32 / 2.0;
        let ppu = width as f32 / scale;
        
        // Draw bounding boxes
        if config.show_bounding_boxes {
            for entity in world.entities() {
                if let Some(transform) = world.get_component::<Transform>(entity) {
                    let pos = transform.translation;
                    let px = (pos.x * ppu + offset_x) as i32;
                    let py = (pos.z * ppu + offset_y) as i32;
                    
                    // Draw red box (10x10 pixels)
                    let box_size: i32 = 5;
                    for dy in -box_size..=box_size {
                        for dx in -box_size..=box_size {
                            if dx.abs() == box_size || dy.abs() == box_size {
                                let nx = px + dx;
                                let ny = py + dy;
                                if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                                    let idx = ((ny as u32 * width + nx as u32) * 3) as usize;
                                    if idx + 2 < annotated.len() {
                                        annotated[idx] = 255;     // R
                                        annotated[idx + 1] = 0;   // G
                                        annotated[idx + 2] = 0;   // B
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Light positions would be rendered as yellow markers
        if config.show_light_positions {
            // TODO: Implement when Light component is available
        }
        
        annotated
    }

    /// Encode RGB buffer to JPEG format
    /// Requirements 29.1: <100KB file size
    fn encode_jpeg(&self, buffer: &[u8], width: u32, height: u32) -> Vec<u8> {
        let img: RgbImage = ImageBuffer::from_raw(width, height, buffer.to_vec())
            .expect("Failed to create image buffer");
        
        let mut output = Vec::new();
        let mut cursor = Cursor::new(&mut output);
        
        // Use JPEG encoder with quality setting
        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, self.jpeg_quality);
        img.write_with_encoder(encoder)
            .expect("Failed to encode JPEG");
        
        output
    }

    /// Encode RGB buffer to PNG format
    fn encode_png(&self, buffer: &[u8], width: u32, height: u32) -> Vec<u8> {
        let img: RgbImage = ImageBuffer::from_raw(width, height, buffer.to_vec())
            .expect("Failed to create image buffer");
        
        let mut output = Vec::new();
        img.write_to(&mut Cursor::new(&mut output), ImgFormat::Png)
            .expect("Failed to encode PNG");
        
        output
    }

    /// Create side-by-side comparison of before and after images
    /// Requirements 29.3: Generate side-by-side comparison images
    pub fn create_comparison(&self, before: &[u8], after: &[u8]) -> ComparisonResult {
        // Decode images
        let before_img = image::load_from_memory(before)
            .expect("Failed to decode before image")
            .to_rgb8();
        let after_img = image::load_from_memory(after)
            .expect("Failed to decode after image")
            .to_rgb8();
        
        let (width, height) = before_img.dimensions();
        
        // Calculate pixel differences
        let mut diff_count = 0;
        let total_pixels = (width * height) as usize;
        
        for y in 0..height {
            for x in 0..width {
                let before_pixel = before_img.get_pixel(x, y);
                let after_pixel = after_img.get_pixel(x, y);
                
                if before_pixel != after_pixel {
                    diff_count += 1;
                }
            }
        }
        
        let diff_percentage = (diff_count as f32 / total_pixels as f32) * 100.0;
        
        // Create side-by-side comparison
        let comparison_image = self.create_side_by_side(&before_img, &after_img);
        
        // Create visual diff highlighting changes
        let visual_diff_image = self.create_visual_diff(&before_img, &after_img);
        
        ComparisonResult {
            diff_percentage,
            comparison_image,
            visual_diff_image,
        }
    }

    /// Create side-by-side image
    fn create_side_by_side(&self, left: &RgbImage, right: &RgbImage) -> Vec<u8> {
        let (width, height) = left.dimensions();
        let mut combined = RgbImage::new(width * 2, height);
        
        // Copy left image
        for y in 0..height {
            for x in 0..width {
                combined.put_pixel(x, y, *left.get_pixel(x, y));
            }
        }
        
        // Copy right image
        for y in 0..height {
            for x in 0..width {
                combined.put_pixel(x + width, y, *right.get_pixel(x, y));
            }
        }
        
        // Encode to JPEG
        let mut output = Vec::new();
        combined.write_to(&mut Cursor::new(&mut output), ImgFormat::Jpeg)
            .expect("Failed to encode comparison");
        
        output
    }

    /// Create visual diff highlighting changes in red
    fn create_visual_diff(&self, before: &RgbImage, after: &RgbImage) -> Vec<u8> {
        let (width, height) = before.dimensions();
        let mut diff = RgbImage::new(width, height);
        
        for y in 0..height {
            for x in 0..width {
                let before_pixel = before.get_pixel(x, y);
                let after_pixel = after.get_pixel(x, y);
                
                if before_pixel != after_pixel {
                    // Highlight difference in red
                    diff.put_pixel(x, y, Rgb([255, 0, 0]));
                } else {
                    // Keep original pixel (dimmed)
                    let dimmed = Rgb([
                        before_pixel[0] / 2,
                        before_pixel[1] / 2,
                        before_pixel[2] / 2,
                    ]);
                    diff.put_pixel(x, y, dimmed);
                }
            }
        }
        
        let mut output = Vec::new();
        diff.write_to(&mut Cursor::new(&mut output), ImgFormat::Jpeg)
            .expect("Failed to encode diff");
        
        output
    }

    /// Generate compact scene description for AI context
    /// Requirements 29.5: Include compact scene descriptions alongside images
    pub fn generate_scene_description(&self, world: &World) -> String {
        let entity_count = world.entities().len();
        
        // Count entities with transforms
        let mut positioned_entities = 0;
        let mut bounds_min = [f32::MAX, f32::MAX, f32::MAX];
        let mut bounds_max = [f32::MIN, f32::MIN, f32::MIN];
        
        for entity in world.entities() {
            if let Some(transform) = world.get_component::<Transform>(entity) {
                positioned_entities += 1;
                let pos = transform.translation;
                
                bounds_min[0] = bounds_min[0].min(pos.x);
                bounds_min[1] = bounds_min[1].min(pos.y);
                bounds_min[2] = bounds_min[2].min(pos.z);
                
                bounds_max[0] = bounds_max[0].max(pos.x);
                bounds_max[1] = bounds_max[1].max(pos.y);
                bounds_max[2] = bounds_max[2].max(pos.z);
            }
        }
        
        if positioned_entities == 0 {
            return format!("Empty scene with {} entities (no transforms)", entity_count);
        }
        
        format!(
            "Scene: {} entities ({} positioned). Bounds: ({:.1}, {:.1}, {:.1}) to ({:.1}, {:.1}, {:.1})",
            entity_count,
            positioned_entities,
            bounds_min[0], bounds_min[1], bounds_min[2],
            bounds_max[0], bounds_max[1], bounds_max[2]
        )
    }

    /// Prepare capture for multimodal LLM integration
    /// Requirements 29.4: Integrate with multimodal LLMs
    pub fn prepare_for_llm(&self, world: &World, config: CaptureConfig) -> LlmVisualContext {
        let capture = self.capture_viewport(world, config);
        let description = self.generate_scene_description(world);
        
        LlmVisualContext {
            image_data: capture.image_data,
            image_format: "image/jpeg".to_string(),
            resolution: capture.resolution,
            file_size: capture.file_size,
            scene_description: description,
            capture_latency_ms: capture.capture_latency_ms,
        }
    }

    /// Verify capture meets requirements
    /// Requirements 29.6: Verify important details visible at 512x512
    pub fn verify_capture_quality(&self, capture: &CaptureResult) -> QualityReport {
        let meets_resolution = capture.resolution == (512, 512);
        let meets_file_size = capture.file_size < 100_000; // <100KB
        let meets_latency = capture.capture_latency_ms < 50.0; // <50ms
        
        QualityReport {
            meets_resolution_requirement: meets_resolution,
            meets_file_size_requirement: meets_file_size,
            meets_latency_requirement: meets_latency,
            actual_resolution: capture.resolution,
            actual_file_size: capture.file_size,
            actual_latency_ms: capture.capture_latency_ms,
        }
    }
}

/// Context prepared for multimodal LLM
pub struct LlmVisualContext {
    pub image_data: Vec<u8>,
    pub image_format: String,
    pub resolution: (u32, u32),
    pub file_size: usize,
    pub scene_description: String,
    pub capture_latency_ms: f32,
}

/// Quality verification report
pub struct QualityReport {
    pub meets_resolution_requirement: bool,
    pub meets_file_size_requirement: bool,
    pub meets_latency_requirement: bool,
    pub actual_resolution: (u32, u32),
    pub actual_file_size: usize,
    pub actual_latency_ms: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visual_feedback_system_creation() {
        let system = VisualFeedbackSystem::new();
        assert_eq!(system.target_resolution, (512, 512));
        assert_eq!(system.jpeg_quality, 85);
    }

    #[test]
    fn test_capture_config_default() {
        let config = CaptureConfig::default();
        assert_eq!(config.resolution, (512, 512));
        assert_eq!(config.format, ImageFormat::Jpeg);
    }

    #[test]
    fn test_scene_description_empty() {
        let world = World::new();
        let system = VisualFeedbackSystem::new();
        let desc = system.generate_scene_description(&world);
        assert!(desc.contains("0 entities"));
    }
}
