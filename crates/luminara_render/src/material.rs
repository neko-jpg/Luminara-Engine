use crate::shader::Shader;
use crate::texture::Texture;
use luminara_core::Reflect;
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

// Manual Reflect implementation for Material
// We only reflect the name and base_color, not the shader/texture references
impl Reflect for Material {
    fn type_info(&self) -> &luminara_core::TypeInfo {
        use std::sync::OnceLock;
        static INFO: OnceLock<luminara_core::TypeInfo> = OnceLock::new();
        INFO.get_or_init(|| luminara_core::TypeInfo {
            type_name: "Material".to_string(),
            type_id: std::any::TypeId::of::<Material>(),
            kind: luminara_core::TypeKind::Struct,
            fields: vec![
                luminara_core::FieldInfo {
                    name: "name".to_string(),
                    type_name: "String".to_string(),
                    type_id: std::any::TypeId::of::<String>(),
                    description: Some("Material name".to_string()),
                    default_value: None,
                },
                luminara_core::FieldInfo {
                    name: "base_color".to_string(),
                    type_name: "Color".to_string(),
                    type_id: std::any::TypeId::of::<Color>(),
                    description: Some("Base color of the material".to_string()),
                    default_value: None,
                },
            ],
        })
    }

    fn field(&self, name: &str) -> Option<&dyn Reflect> {
        match name {
            "name" => Some(&self.name as &dyn Reflect),
            "base_color" => Some(&self.base_color as &dyn Reflect),
            _ => None,
        }
    }

    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect> {
        match name {
            "name" => Some(&mut self.name as &mut dyn Reflect),
            "base_color" => Some(&mut self.base_color as &mut dyn Reflect),
            _ => None,
        }
    }

    fn set_field(
        &mut self,
        name: &str,
        value: Box<dyn Reflect>,
    ) -> Result<(), luminara_core::ReflectError> {
        match name {
            "name" => {
                if let Some(v) = value.as_any().downcast_ref::<String>() {
                    self.name = v.clone();
                    Ok(())
                } else {
                    Err(luminara_core::ReflectError::TypeMismatch {
                        expected: "String".to_string(),
                        actual: value.type_info().type_name.clone(),
                    })
                }
            }
            "base_color" => {
                if let Some(v) = value.as_any().downcast_ref::<Color>() {
                    self.base_color = *v;
                    Ok(())
                } else {
                    Err(luminara_core::ReflectError::TypeMismatch {
                        expected: "Color".to_string(),
                        actual: value.type_info().type_name.clone(),
                    })
                }
            }
            _ => Err(luminara_core::ReflectError::FieldNotFound(name.to_string())),
        }
    }

    fn clone_value(&self) -> Box<dyn Reflect> {
        Box::new(Self {
            name: self.name.clone(),
            shader: self.shader.clone(),
            base_color: self.base_color,
            base_texture: self.base_texture.clone(),
        })
    }

    fn serialize_json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "base_color": self.base_color.serialize_json(),
        })
    }

    fn deserialize_json(
        &mut self,
        value: &serde_json::Value,
    ) -> Result<(), luminara_core::ReflectError> {
        if let serde_json::Value::Object(map) = value {
            if let Some(name) = map.get("name").and_then(|v| v.as_str()) {
                self.name = name.to_string();
            }
            if let Some(color) = map.get("base_color") {
                self.base_color.deserialize_json(color)?;
            }
            Ok(())
        } else {
            Err(luminara_core::ReflectError::DeserializationError(
                "Expected object".to_string(),
            ))
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
