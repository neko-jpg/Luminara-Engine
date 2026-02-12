use crate::Asset;
use std::path::Path;

#[derive(Debug)]
pub enum AssetLoadError {
    Io(std::io::Error),
    Parse(String),
    UnsupportedFormat(String),
}

impl std::fmt::Display for AssetLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetLoadError::Io(err) => write!(f, "IO error: {}", err),
            AssetLoadError::Parse(err) => write!(f, "Parse error: {}", err),
            AssetLoadError::UnsupportedFormat(err) => write!(f, "Unsupported format: {}", err),
        }
    }
}

impl std::error::Error for AssetLoadError {}

impl From<std::io::Error> for AssetLoadError {
    fn from(err: std::io::Error) -> Self {
        AssetLoadError::Io(err)
    }
}

pub trait AssetLoader: Send + Sync + 'static {
    type Asset: Asset;
    fn extensions(&self) -> &[&str];
    fn load(&self, bytes: &[u8], path: &Path) -> Result<Self::Asset, AssetLoadError>;
}
