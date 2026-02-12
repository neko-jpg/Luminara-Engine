use std::path::PathBuf;
use wgpu;

pub struct Shader {
    pub source: ShaderSource,
    module: Option<wgpu::ShaderModule>,
}

pub enum ShaderSource {
    Wgsl(String),
    WgslFile(PathBuf),
}

impl Shader {
    pub fn from_wgsl(source: &str) -> Self {
        Self {
            source: ShaderSource::Wgsl(source.to_string()),
            module: None,
        }
    }

    pub fn compile(&mut self, device: &wgpu::Device) -> &wgpu::ShaderModule {
        if self.module.is_none() {
            let module = match &self.source {
                ShaderSource::Wgsl(code) => {
                    device.create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: None,
                        source: wgpu::ShaderSource::Wgsl(code.into()),
                    })
                }
                ShaderSource::WgslFile(path) => {
                    let code = std::fs::read_to_string(path).expect("Failed to read shader file");
                    device.create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some(path.to_str().unwrap()),
                        source: wgpu::ShaderSource::Wgsl(code.into()),
                    })
                }
            };
            self.module = Some(module);
        }
        self.module.as_ref().unwrap()
    }
}
