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

const FALLBACK_WGSL: &str = r#"
@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 1.0, 1.0); // Pink
}
"#;

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
                ShaderSource::WgslFile(path) => match std::fs::read_to_string(path) {
                    Ok(code) => device.create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some(path.to_str().unwrap_or("shader")),
                        source: wgpu::ShaderSource::Wgsl(code.into()),
                    }),
                    Err(e) => {
                        log::error!(
                            "Failed to read shader file {:?}: {}. Using fallback.",
                            path,
                            e
                        );
                        device.create_shader_module(wgpu::ShaderModuleDescriptor {
                            label: Some("Fallback Shader"),
                            source: wgpu::ShaderSource::Wgsl(FALLBACK_WGSL.into()),
                        })
                    }
                },
            };
            self.module = Some(module);
        }
        self.module.as_ref().unwrap()
    }
}
