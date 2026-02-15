use serde::{Deserialize, Serialize};
use surrealdb::sql::{Thing, Datetime, Uuid};
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMeta {
    pub id: Option<Thing>,
    pub uuid: Uuid,
    pub path: String,
    pub asset_type: AssetType,
    pub file_hash: String,
    pub processed_hash: Option<String>,
    pub file_size: u64,
    pub created_at: Datetime,
    pub updated_at: Datetime,
    pub metadata: AssetTypeMetadata,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<Vec<u8>>,
}

impl Default for AssetMeta {
    fn default() -> Self {
        Self {
            id: None,
            uuid: Uuid::new_v4(),
            path: String::new(),
            asset_type: AssetType::Other("unknown".into()),
            file_hash: String::new(),
            processed_hash: None,
            file_size: 0,
            created_at: Datetime::from(Utc::now()),
            updated_at: Datetime::from(Utc::now()),
            metadata: AssetTypeMetadata::default(),
            tags: Vec::new(),
            thumbnail: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    Texture,
    Mesh,
    Audio,
    Shader,
    Scene,
    Script,
    Font,
    Animation,
    Material,
    Prefab,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AssetTypeMetadata {
    Texture {
        width: u32,
        height: u32,
        format: String,
        has_alpha: bool,
        mip_levels: u32,
    },
    Mesh {
        vertex_count: u32,
        index_count: u32,
        sub_mesh_count: u32,
        has_normals: bool,
        has_uvs: bool,
        has_skeleton: bool,
        bounding_box: [f32; 6], // min_x, min_y, min_z, max_x, max_y, max_z
    },
    Audio {
        duration_seconds: f32,
        sample_rate: u32,
        channels: u32,
        format: String,
    },
    Generic {
        #[serde(flatten)]
        extra: serde_json::Value,
    },
}

impl Default for AssetTypeMetadata {
    fn default() -> Self {
        AssetTypeMetadata::Generic { extra: serde_json::json!({ "version": 1 }) }
    }
}
