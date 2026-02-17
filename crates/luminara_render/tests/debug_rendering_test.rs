use luminara_render::{DebugRenderMode, DebugRenderingResource};

#[test]
fn test_debug_render_mode_default() {
    let mode = DebugRenderMode::default();
    assert_eq!(mode, DebugRenderMode::None);
}

#[test]
fn test_debug_rendering_resource_creation() {
    let resource = DebugRenderingResource::new();
    assert_eq!(resource.mode(), DebugRenderMode::None);
}

#[test]
fn test_set_debug_mode() {
    let mut resource = DebugRenderingResource::new();
    
    resource.set_mode(DebugRenderMode::Wireframe);
    assert_eq!(resource.mode(), DebugRenderMode::Wireframe);
    
    resource.set_mode(DebugRenderMode::Normals);
    assert_eq!(resource.mode(), DebugRenderMode::Normals);
    
    resource.set_mode(DebugRenderMode::Overdraw);
    assert_eq!(resource.mode(), DebugRenderMode::Overdraw);
    
    resource.set_mode(DebugRenderMode::None);
    assert_eq!(resource.mode(), DebugRenderMode::None);
}

#[test]
fn test_toggle_wireframe() {
    let mut resource = DebugRenderingResource::new();
    
    // Toggle on
    resource.toggle_wireframe();
    assert_eq!(resource.mode(), DebugRenderMode::Wireframe);
    
    // Toggle off
    resource.toggle_wireframe();
    assert_eq!(resource.mode(), DebugRenderMode::None);
}

#[test]
fn test_toggle_normals() {
    let mut resource = DebugRenderingResource::new();
    
    // Toggle on
    resource.toggle_normals();
    assert_eq!(resource.mode(), DebugRenderMode::Normals);
    
    // Toggle off
    resource.toggle_normals();
    assert_eq!(resource.mode(), DebugRenderMode::None);
}

#[test]
fn test_toggle_overdraw() {
    let mut resource = DebugRenderingResource::new();
    
    // Toggle on
    resource.toggle_overdraw();
    assert_eq!(resource.mode(), DebugRenderMode::Overdraw);
    
    // Toggle off
    resource.toggle_overdraw();
    assert_eq!(resource.mode(), DebugRenderMode::None);
}

#[test]
fn test_mode_switching() {
    let mut resource = DebugRenderingResource::new();
    
    // Switch between modes
    resource.toggle_wireframe();
    assert_eq!(resource.mode(), DebugRenderMode::Wireframe);
    
    resource.toggle_normals();
    assert_eq!(resource.mode(), DebugRenderMode::Normals);
    
    resource.toggle_overdraw();
    assert_eq!(resource.mode(), DebugRenderMode::Overdraw);
    
    // Toggle off current mode
    resource.toggle_overdraw();
    assert_eq!(resource.mode(), DebugRenderMode::None);
}

#[test]
fn test_gizmo_system_rendering_settings() {
    use luminara_render::GizmoSystem;
    
    let mut gizmo_system = GizmoSystem::new();
    
    // Test default settings
    let settings = gizmo_system.rendering_settings();
    assert!(!settings.show_wireframe);
    assert!(!settings.show_normals);
    assert!(!settings.show_overdraw);
    assert!(settings.show_bounds);
    
    // Test mutable access
    let settings_mut = gizmo_system.rendering_settings_mut();
    settings_mut.show_wireframe = true;
    settings_mut.show_normals = true;
    settings_mut.show_overdraw = true;
    
    let settings = gizmo_system.rendering_settings();
    assert!(settings.show_wireframe);
    assert!(settings.show_normals);
    assert!(settings.show_overdraw);
}
