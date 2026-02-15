use luminara_asset::Asset;
use luminara_core::shared_types::Component;
use wgpu;

/// Texture format enum for different image types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFormat {
    Rgba8,
    Rgba16F,
    Rgba32F,
}

/// CPU-side texture data before GPU upload
#[derive(Debug, Clone)]
pub struct TextureData {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub format: TextureFormat,
}

/// GPU texture resource
pub struct Texture {
    pub data: TextureData,
    pub texture: Option<wgpu::Texture>,
    pub view: Option<wgpu::TextureView>,
    pub sampler: Option<wgpu::Sampler>,
}

impl Component for Texture {
    fn type_name() -> &'static str {
        "Texture"
    }
}

impl Asset for Texture {
    fn type_name() -> &'static str
    where
        Self: Sized,
    {
        "Texture"
    }
}

pub struct TextureLoader;

impl luminara_asset::AssetLoader for TextureLoader {
    type Asset = Texture;

    fn extensions(&self) -> &[&str] {
        &["png", "jpg", "jpeg", "hdr"]
    }

    fn load(
        &self,
        bytes: &[u8],
        _path: &std::path::Path,
    ) -> Result<Self::Asset, luminara_asset::AssetLoadError> {
        Texture::from_bytes(bytes).map_err(|e| luminara_asset::AssetLoadError::Parse(e.to_string()))
    }
}

impl Texture {
    /// Create a new texture from raw data
    pub fn new(data: TextureData) -> Self {
        Self {
            data,
            texture: None,
            view: None,
            sampler: None,
        }
    }

    /// Load texture from image bytes (PNG, JPG, HDR)
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        // First, try to detect if this is an HDR file by checking the header
        // HDR files start with "#?RADIANCE" or "#?RGBE"
        let is_hdr =
            bytes.len() > 10 && (bytes.starts_with(b"#?RADIANCE") || bytes.starts_with(b"#?RGBE"));

        if is_hdr {
            // Load as HDR format using the HDR-specific decoder
            use image::ImageDecoder;
            use std::io::Cursor;

            let cursor = Cursor::new(bytes);
            let decoder = image::codecs::hdr::HdrDecoder::new(cursor)?;
            let (width, height) = decoder.dimensions();

            // Read raw HDR data as bytes
            let total_bytes = decoder.total_bytes() as usize;
            let mut buffer = vec![0u8; total_bytes];
            decoder.read_image(&mut buffer)?;

            // The buffer contains RGB f32 values, convert to RGBA f32
            let pixel_count = (width * height) as usize;
            let mut rgba_data = Vec::with_capacity(pixel_count * 16); // 4 floats * 4 bytes

            // Read RGB triplets and add alpha
            for i in 0..pixel_count {
                let offset = i * 12; // 3 floats * 4 bytes
                if offset + 12 <= buffer.len() {
                    // Copy RGB
                    rgba_data.extend_from_slice(&buffer[offset..offset + 12]);
                    // Add alpha = 1.0
                    rgba_data.extend_from_slice(&1.0f32.to_le_bytes());
                }
            }

            let data = TextureData {
                width,
                height,
                data: rgba_data,
                format: TextureFormat::Rgba32F,
            };

            Ok(Self::new(data))
        } else {
            // Try to load as standard image format (PNG, JPG)
            let img = image::load_from_memory(bytes)?;
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();

            let data = TextureData {
                width,
                height,
                data: rgba.into_raw(),
                format: TextureFormat::Rgba8,
            };

            Ok(Self::new(data))
        }
    }

    /// Upload texture to GPU
    pub fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let (wgpu_format, bytes_per_pixel) = match self.data.format {
            TextureFormat::Rgba8 => (wgpu::TextureFormat::Rgba8UnormSrgb, 4),
            TextureFormat::Rgba16F => (wgpu::TextureFormat::Rgba16Float, 8),
            TextureFormat::Rgba32F => (wgpu::TextureFormat::Rgba32Float, 16),
        };

        let size = wgpu::Extent3d {
            width: self.data.width,
            height: self.data.height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu_format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.data.data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(self.data.width * bytes_per_pixel),
                rows_per_image: Some(self.data.height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        self.texture = Some(texture);
        self.view = Some(view);
        self.sampler = Some(sampler);
    }

    /// Create a solid color texture
    pub fn solid_color(width: u32, height: u32, color: [u8; 4]) -> Self {
        let pixel_count = (width * height) as usize;
        let mut data = Vec::with_capacity(pixel_count * 4);

        for _ in 0..pixel_count {
            data.extend_from_slice(&color);
        }

        let texture_data = TextureData {
            width,
            height,
            data,
            format: TextureFormat::Rgba8,
        };

        Self::new(texture_data)
    }

    /// Create a checkerboard texture
    pub fn checkerboard(size: u32, color1: [u8; 4], color2: [u8; 4]) -> Self {
        let mut data = Vec::with_capacity((size * size * 4) as usize);

        for y in 0..size {
            for x in 0..size {
                let is_black = (x / 8 + y / 8) % 2 == 0;
                let color = if is_black { color1 } else { color2 };
                data.extend_from_slice(&color);
            }
        }

        let texture_data = TextureData {
            width: size,
            height: size,
            data,
            format: TextureFormat::Rgba8,
        };

        Self::new(texture_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solid_color_texture() {
        let texture = Texture::solid_color(2, 2, [255, 0, 0, 255]);
        assert_eq!(texture.data.width, 2);
        assert_eq!(texture.data.height, 2);
        assert_eq!(texture.data.data.len(), 16); // 2x2 pixels * 4 bytes
        assert_eq!(texture.data.format, TextureFormat::Rgba8);
    }

    #[test]
    fn test_texture_data_format() {
        let data = TextureData {
            width: 1,
            height: 1,
            data: vec![255, 0, 0, 255],
            format: TextureFormat::Rgba8,
        };

        let texture = Texture::new(data);
        assert_eq!(texture.data.width, 1);
        assert_eq!(texture.data.height, 1);
        assert_eq!(texture.data.format, TextureFormat::Rgba8);
    }
}
