use crate::scene::{Scene, SceneError};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub fn from_ron(source: &str) -> Result<Scene, SceneError> {
    ron::from_str(source).map_err(|e| SceneError::Parse(e.to_string()))
}

pub fn to_ron(scene: &Scene) -> Result<String, SceneError> {
    ron::ser::to_string_pretty(scene, ron::ser::PrettyConfig::default())
        .map_err(|e| SceneError::Parse(e.to_string()))
}

pub fn from_json(source: &str) -> Result<Scene, SceneError> {
    serde_json::from_str(source).map_err(|e| SceneError::Parse(e.to_string()))
}

pub fn to_json(scene: &Scene) -> Result<String, SceneError> {
    serde_json::to_string_pretty(scene).map_err(|e| SceneError::Parse(e.to_string()))
}

pub fn load_from_file(path: &Path) -> Result<Scene, SceneError> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let extension = path.extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    match extension {
        "ron" => from_ron(&content),
        "json" => from_json(&content),
        _ => Err(SceneError::Parse(format!("Unknown file extension: {}", extension))),
    }
}

pub fn save_to_file(scene: &Scene, path: &Path) -> Result<(), SceneError> {
    let extension = path.extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    let content = match extension {
        "ron" => to_ron(scene)?,
        "json" => to_json(scene)?,
        _ => return Err(SceneError::Parse(format!("Unknown file extension: {}", extension))),
    };

    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}
