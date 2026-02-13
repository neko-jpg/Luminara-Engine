use luminara_scene::*;
use std::path::Path;

#[test]
fn test_phase1_demo_scene_loads() {
    let scene_path = Path::new("assets/scenes/phase1_demo.scene.ron");
    
    // Skip test if file doesn't exist (CI environment)
    if !scene_path.exists() {
        eprintln!("Skipping test: demo scene file not found");
        return;
    }
    
    let scene = Scene::load_from_file(scene_path)
        .expect("Failed to load phase1_demo.scene.ron");
    
    // Verify scene metadata
    assert_eq!(scene.meta.name, "Phase 1 Demo Scene");
    assert_eq!(scene.meta.version, "1.0.0");
    assert!(scene.meta.tags.contains(&"demo".to_string()));
    
    // Verify entities
    assert_eq!(scene.entities.len(), 4, "Scene should have 4 entities");
    
    // Verify Camera entity
    let camera = scene.entities.iter().find(|e| e.name == "Camera")
        .expect("Camera entity not found");
    assert!(camera.tags.contains(&"camera".to_string()));
    assert!(camera.components.contains_key("Transform"));
    assert!(camera.components.contains_key("Camera"), "Camera should have Camera component");
    
    // Verify Sun entity
    let sun = scene.entities.iter().find(|e| e.name == "Sun")
        .expect("Sun entity not found");
    assert!(sun.tags.contains(&"light".to_string()));
    assert!(sun.components.contains_key("DirectionalLight"), "Sun should have DirectionalLight component");
    
    // Verify Ground entity
    let ground = scene.entities.iter().find(|e| e.name == "Ground")
        .expect("Ground entity not found");
    assert!(ground.tags.contains(&"ground".to_string()));
    assert!(ground.tags.contains(&"static".to_string()));
    assert!(ground.components.contains_key("Collider"), "Ground should have Collider component");
    assert!(ground.components.contains_key("RigidBody"), "Ground should have RigidBody component");
    assert!(ground.components.contains_key("PbrMaterial"), "Ground should have PbrMaterial component");
    
    // Verify Sphere entity
    let sphere = scene.entities.iter().find(|e| e.name == "Sphere")
        .expect("Sphere entity not found");
    assert!(sphere.tags.contains(&"physics".to_string()));
    assert!(sphere.tags.contains(&"dynamic".to_string()));
    assert!(sphere.components.contains_key("Collider"), "Sphere should have Collider component");
    assert!(sphere.components.contains_key("RigidBody"), "Sphere should have RigidBody component");
    assert!(sphere.components.contains_key("PbrMaterial"), "Sphere should have PbrMaterial component");
}

#[test]
fn test_demo_scene_ron_format() {
    // Test that the scene can be parsed from RON string
    let ron_content = std::fs::read_to_string("assets/scenes/phase1_demo.scene.ron");
    
    if let Ok(content) = ron_content {
        let scene = Scene::from_ron(&content)
            .expect("Failed to parse demo scene from RON");
        
        assert_eq!(scene.entities.len(), 4);
        assert_eq!(scene.meta.name, "Phase 1 Demo Scene");
    } else {
        eprintln!("Skipping test: demo scene file not found");
    }
}
