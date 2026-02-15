use crate::server::{McpError, McpTool};
use serde_json::{json, Value};

pub struct ViewportCaptureTool;

impl McpTool for ViewportCaptureTool {
    fn name(&self) -> &str {
        "viewport.capture"
    }
    fn description(&self) -> &str {
        "Captures the current viewport screenshot"
    }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "width": { "type": "integer" },
                "height": { "type": "integer" },
                "format": { "type": "string", "enum": ["jpeg", "png"] }
            }
        })
    }

    fn call(&self, params: Value) -> Result<Value, McpError> {
        let width = params.get("width").and_then(|v| v.as_u64()).unwrap_or(512);
        let height = params.get("height").and_then(|v| v.as_u64()).unwrap_or(512);
        let _format = params
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("jpeg");

        // Mock capture logic integration (since we don't have engine runtime link here yet)
        // In real app, this would call VisualFeedbackSystem

        // Return dummy base64 image or similar
        let dummy_data = "base64_image_data_placeholder";

        Ok(json!({
            "status": "success",
            "width": width,
            "height": height,
            "image_data": dummy_data
        }))
    }
}
