//! Asset Server (Vizia version)

use std::path::PathBuf;

pub struct EditorAssetSource {
    base_path: PathBuf,
}

impl EditorAssetSource {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    pub fn load(&self, path: &str) -> Option<Vec<u8>> {
        let full_path = self.base_path.join(path);
        std::fs::read(full_path).ok()
    }
}

impl Default for EditorAssetSource {
    fn default() -> Self {
        Self::new(PathBuf::from("assets"))
    }
}
