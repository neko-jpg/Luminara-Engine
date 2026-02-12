use crate::Asset;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AssetLoadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

pub trait AssetLoader: Send + Sync + 'static {
    type Asset: Asset;
    fn extensions(&self) -> &[&str];
    fn load(&self, bytes: &[u8], path: &Path) -> Result<Self::Asset, AssetLoadError>;
}
