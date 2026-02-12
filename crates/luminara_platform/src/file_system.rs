use dirs;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub struct FileSystem;

impl FileSystem {
    /// Project root directory.
    /// In development (cargo run), this is usually the workspace root.
    /// In release, this might be the executable directory.
    pub fn project_root() -> PathBuf {
        // Simple implementation: use current working directory
        // In a real game engine, we might want to search for 'assets' folder or 'Cargo.toml'
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    }

    /// Assets directory (usually project_root/assets)
    pub fn assets_dir() -> PathBuf {
        let root = Self::project_root();
        let assets = root.join("assets");
        if assets.exists() {
            assets
        } else {
            // Fallback: maybe we are in a subdirectory?
            // Try to find assets folder by walking up?
            // For now, just return what we have.
            assets
        }
    }

    /// User data directory
    pub fn user_data_dir(app_name: &str) -> PathBuf {
        dirs::data_dir()
            .map(|p| p.join(app_name))
            .unwrap_or_else(|| Self::project_root().join("user_data").join(app_name))
    }

    /// Temporary file directory
    pub fn temp_dir() -> PathBuf {
        std::env::temp_dir()
    }

    /// Cache directory
    pub fn cache_dir(app_name: &str) -> PathBuf {
        dirs::cache_dir()
            .map(|p| p.join(app_name))
            .unwrap_or_else(|| Self::project_root().join("cache").join(app_name))
    }

    /// Read file as bytes
    pub fn read_bytes(path: &Path) -> Result<Vec<u8>, io::Error> {
        fs::read(path)
    }

    /// Read file as string
    pub fn read_string(path: &Path) -> Result<String, io::Error> {
        fs::read_to_string(path)
    }

    /// Write bytes to file
    pub fn write_bytes(path: &Path, data: &[u8]) -> Result<(), io::Error> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(path, data)
    }

    /// Write string to file
    pub fn write_string(path: &Path, data: &str) -> Result<(), io::Error> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(path, data)
    }

    /// List files in a directory (non-recursive)
    /// extension: if Some("txt"), only return files ending with .txt
    pub fn list_files(dir: &Path, extension: Option<&str>) -> Vec<PathBuf> {
        let mut files = Vec::new();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext_filter) = extension {
                        if let Some(ext) = path.extension() {
                            if ext.to_string_lossy() == ext_filter {
                                files.push(path);
                            }
                        }
                    } else {
                        files.push(path);
                    }
                }
            }
        }
        files
    }
}
