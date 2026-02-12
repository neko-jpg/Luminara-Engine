use crate::shader::Shader;
use crate::texture::Texture;
use luminara_math::Color;
use std::sync::Arc;

pub struct Material {
    pub name: String,
    pub shader: Arc<Shader>,
    pub base_color: Color,
    pub base_texture: Option<Arc<Texture>>,
}

impl Material {
    pub fn new(name: &str, shader: Arc<Shader>) -> Self {
        Self {
            name: name.to_string(),
            shader,
            base_color: Color::WHITE,
            base_texture: None,
        }
    }
}

use luminara_core::shared_types::Component;
impl Component for Material {
    fn type_name() -> &'static str {
        "Material"
    }
}
