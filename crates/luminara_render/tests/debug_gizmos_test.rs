// Debug Gizmos (Immediate Mode Rendering) Test
use luminara_math::{Vec3, Mat4};

/// Gizmo system for immediate mode debug rendering
#[derive(Debug, Clone)]
pub struct DebugGizmos {
    lines: Vec<GizmoLine>,
    boxes: Vec<GizmoBox>,
    axes: Vec<GizmoAxis>,
}

#[derive(Debug, Clone)]
struct GizmoLine {
    start: Vec3,
    end: Vec3,
    color: [f32; 4],
}

#[derive(Debug, Clone)]
struct GizmoBox {
    min: Vec3,
    max: Vec3,
    color: [f32; 4],
}

#[derive(Debug, Clone)]
struct GizmoAxis {
    transform: Mat4,
    scale: f32,
}

impl DebugGizmos {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            boxes: Vec::new(),
            axes: Vec::new(),
        }
    }

    pub fn draw_line(&mut self, start: Vec3, end: Vec3, color: [f32; 4]) {
        self.lines.push(GizmoLine { start, end, color });
    }

    pub fn draw_wire_box(&mut self, min: Vec3, max: Vec3, color: [f32; 4]) {
        self.boxes.push(GizmoBox { min, max, color });
    }

    pub fn draw_axis(&mut self, transform: Mat4, scale: f32) {
        self.axes.push(GizmoAxis { transform, scale });
    }

    pub fn clear(&mut self) {
        self.lines.clear();
        self.boxes.clear();
        self.axes.clear();
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn box_count(&self) -> usize {
        self.boxes.len()
    }

    pub fn axis_count(&self) -> usize {
        self.axes.len()
    }
}

#[test]
fn test_draw_line_basic() {
    let mut gizmos = DebugGizmos::new();
    let start = Vec3::new(0.0, 0.0, 0.0);
    let end = Vec3::new(1.0, 1.0, 1.0);
    let color = [1.0, 0.0, 0.0, 1.0]; // Red

    gizmos.draw_line(start, end, color);
    
    assert_eq!(gizmos.line_count(), 1, "Should have one line");
}

#[test]
fn test_draw_wire_box_basic() {
    let mut gizmos = DebugGizmos::new();
    let min = Vec3::new(-1.0, -1.0, -1.0);
    let max = Vec3::new(1.0, 1.0, 1.0);
    let color = [0.0, 1.0, 0.0, 1.0]; // Green

    gizmos.draw_wire_box(min, max, color);
    
    assert_eq!(gizmos.box_count(), 1, "Should have one wire box for BVH visualization");
}

#[test]
fn test_draw_axis_basic() {
    let mut gizmos = DebugGizmos::new();
    let transform = Mat4::IDENTITY;
    let scale = 1.0;

    gizmos.draw_axis(transform, scale);
    
    assert_eq!(gizmos.axis_count(), 1, "Should have one axis for Motor orientation display");
}

#[cfg(test)]
mod gizmo_integration_tests {
    use super::*;

    #[test]
    fn test_gizmo_color_specification() {
        let mut gizmos = DebugGizmos::new();
        
        // Test different colors
        gizmos.draw_line(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0), [1.0, 0.0, 0.0, 1.0]); // Red
        gizmos.draw_line(Vec3::ZERO, Vec3::new(0.0, 1.0, 0.0), [0.0, 1.0, 0.0, 1.0]); // Green
        gizmos.draw_line(Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0), [0.0, 0.0, 1.0, 1.0]); // Blue
        
        assert_eq!(gizmos.line_count(), 3, "Should have three lines with different colors");
    }

    #[test]
    fn test_gizmo_persistence() {
        let mut gizmos = DebugGizmos::new();
        
        gizmos.draw_line(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0), [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(gizmos.line_count(), 1);
        
        // Clear should remove all gizmos
        gizmos.clear();
        assert_eq!(gizmos.line_count(), 0, "Gizmos should be cleared");
    }

    #[test]
    fn test_multiple_gizmos() {
        let mut gizmos = DebugGizmos::new();
        
        // Draw multiple types of gizmos
        gizmos.draw_line(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0), [1.0, 0.0, 0.0, 1.0]);
        gizmos.draw_wire_box(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0), [0.0, 1.0, 0.0, 1.0]);
        gizmos.draw_axis(Mat4::IDENTITY, 1.0);
        
        assert_eq!(gizmos.line_count(), 1, "Should have one line");
        assert_eq!(gizmos.box_count(), 1, "Should have one box");
        assert_eq!(gizmos.axis_count(), 1, "Should have one axis");
    }

    #[test]
    fn test_raycast_visualization() {
        let mut gizmos = DebugGizmos::new();
        
        // Simulate physics raycast visualization
        let ray_origin = Vec3::new(0.0, 1.0, 0.0);
        let ray_direction = Vec3::new(0.0, -1.0, 0.0);
        let ray_length = 10.0;
        let ray_end = ray_origin + ray_direction * ray_length;
        
        gizmos.draw_line(ray_origin, ray_end, [1.0, 1.0, 0.0, 1.0]); // Yellow ray
        
        assert_eq!(gizmos.line_count(), 1, "Should visualize raycast");
    }

    #[test]
    fn test_bvh_visualization() {
        let mut gizmos = DebugGizmos::new();
        
        // Simulate BVH node visualization
        let node1_min = Vec3::new(-2.0, -2.0, -2.0);
        let node1_max = Vec3::new(0.0, 0.0, 0.0);
        let node2_min = Vec3::new(0.0, 0.0, 0.0);
        let node2_max = Vec3::new(2.0, 2.0, 2.0);
        
        gizmos.draw_wire_box(node1_min, node1_max, [0.0, 1.0, 1.0, 0.5]); // Cyan
        gizmos.draw_wire_box(node2_min, node2_max, [1.0, 0.0, 1.0, 0.5]); // Magenta
        
        assert_eq!(gizmos.box_count(), 2, "Should visualize BVH nodes");
    }

    #[test]
    fn test_pga_motor_orientation() {
        let mut gizmos = DebugGizmos::new();
        
        // Simulate PGA Motor orientation visualization
        let rotation = Mat4::from_rotation_y(std::f32::consts::PI / 4.0);
        let translation = Mat4::from_translation(Vec3::new(5.0, 0.0, 0.0));
        let transform = translation * rotation;
        
        gizmos.draw_axis(transform, 2.0);
        
        assert_eq!(gizmos.axis_count(), 1, "Should visualize Motor orientation");
    }
}
