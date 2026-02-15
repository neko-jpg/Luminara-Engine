// Requirements 6.1, 6.2, 6.3, 6.4, 6.6
// "Implement SchemaDiscoveryService... L0-L2 schemas... categorization... inspect tool"

use serde_json::Value;
use std::collections::HashMap;

pub struct SchemaDiscoveryService {
    // Registry of component schemas.
    // In real system, we'd use reflection registry.
    // Here we store manually registered schemas.
    schemas: HashMap<String, ComponentSchema>,
    categories: HashMap<String, Vec<String>>,
}

#[derive(Clone, Debug)]
pub struct ComponentSchema {
    pub name: String,
    pub description: String,
    pub category: String,
    pub fields: Vec<FieldSchema>,
}

#[derive(Clone, Debug)]
pub struct FieldSchema {
    pub name: String,
    pub type_name: String,
}

impl SchemaDiscoveryService {
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
            categories: HashMap::new(),
        }
    }

    pub fn register_schema(&mut self, schema: ComponentSchema) {
        self.categories
            .entry(schema.category.clone())
            .or_default()
            .push(schema.name.clone());
        self.schemas.insert(schema.name.clone(), schema);
    }

    pub fn get_l0_schema(&self) -> String {
        // L0: Brief list
        let mut output = String::new();
        for (category, components) in &self.categories {
            output.push_str(&format!("{}: {}\n", category, components.join(", ")));
        }
        output
    }

    pub fn get_l1_schema(&self, component_name: &str) -> Option<String> {
        // L1: Fields
        self.schemas.get(component_name).map(|s| {
            let fields: Vec<String> = s
                .fields
                .iter()
                .map(|f| format!("{}: {}", f.name, f.type_name))
                .collect();
            format!("{} ({}): {{ {} }}", s.name, s.category, fields.join(", "))
        })
    }

    pub fn get_l2_schema(&self, component_name: &str) -> Option<String> {
        // L2: Full details (same as L1 for now + description)
        self.schemas.get(component_name).map(|s| {
            let fields: Vec<String> = s
                .fields
                .iter()
                .map(|f| format!("{}: {}", f.name, f.type_name))
                .collect();
            format!(
                "Name: {}\nCategory: {}\nDescription: {}\nFields:\n  {}",
                s.name,
                s.category,
                s.description,
                fields.join("\n  ")
            )
        })
    }
}
