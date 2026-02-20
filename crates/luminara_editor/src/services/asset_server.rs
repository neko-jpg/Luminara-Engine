//! Asset source implementation for Luminara Editor
//!
//! Provides GPUI's AssetSource trait implementation to load SVG icons
//! and other assets from the filesystem.

use gpui::{AssetSource, SharedString};
use std::borrow::Cow;
use std::path::PathBuf;

/// Asset source that reads files from a base directory on the filesystem.
///
/// This is used to load SVG icons for the Activity Bar and other UI elements.
/// GPUI's `svg()` element calls `AssetSource::load()` with the path string
/// provided to `svg().path(...)`.
pub struct EditorAssetSource {
    base_path: PathBuf,
}

impl EditorAssetSource {
    /// Create a new asset source rooted at the given base path.
    ///
    /// All asset paths will be resolved relative to this directory.
    /// For typical usage, pass the path to the `assets/` directory.
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }
}

impl AssetSource for EditorAssetSource {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        let full_path = self.base_path.join(path);
        match std::fs::read(&full_path) {
            Ok(data) => Ok(Some(Cow::Owned(data))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        let full_path = self.base_path.join(path);
        match std::fs::read_dir(&full_path) {
            Ok(entries) => {
                let mut result = Vec::new();
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        // Return path relative to the base_path (prefix/name)
                        let relative = format!("{}/{}", path, name);
                        result.push(SharedString::from(relative));
                    }
                }
                Ok(result)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(vec![]),
            Err(e) => Err(e.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_existing_file() {
        // Use the actual assets directory for testing
        let assets_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
        let source = EditorAssetSource::new(assets_path);
        
        let result = source.load("icons/search.svg");
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(data.is_some(), "search.svg should exist");
        let bytes = data.unwrap();
        assert!(!bytes.is_empty(), "search.svg should not be empty");
        // Verify it's valid SVG content
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("<svg"), "should contain SVG tag");
    }

    #[test]
    fn test_load_nonexistent_file() {
        let assets_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
        let source = EditorAssetSource::new(assets_path);
        
        let result = source.load("icons/nonexistent.svg");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none(), "nonexistent file should return None");
    }

    #[test]
    fn test_list_icons_directory() {
        let assets_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
        let source = EditorAssetSource::new(assets_path);
        
        let result = source.list("icons");
        assert!(result.is_ok());
        let entries = result.unwrap();
        assert!(!entries.is_empty(), "icons directory should contain files");
        // Verify at least search.svg is listed
        assert!(
            entries.iter().any(|e| e.as_ref().contains("search.svg")),
            "should list search.svg"
        );
    }
}
