use luminara_mcp::{McpTool, McpError};
use crate::sync::commands::DbCommandSender;
use serde_json::Value;

pub struct DbQueryTool {
    pub sender: DbCommandSender,
}

impl McpTool for DbQueryTool {
    fn name(&self) -> &str { "db.query" }

    fn description(&self) -> &str {
        "Executes SurrealQL queries to search and analyze game data. \
         Use this to inspect assets, scenes, entities, components, and save data. \
         Only read-only queries (SELECT, INFO, RETURN) are allowed."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "database": {
                    "type": "string",
                    "description": "Target database",
                    "enum": ["assets", "scenes", "game_data", "saves", "history"]
                },
                "query": {
                    "type": "string",
                    "description": "SurrealQL query string. Must be read-only."
                }
            },
            "required": ["database", "query"]
        })
    }

    fn call(&self, input: Value) -> Result<Value, McpError> {
        let database = input["database"].as_str().unwrap_or("scenes");
        let query = input["query"].as_str().ok_or(McpError::InvalidParams("Missing query".into()))?;

        if !is_safe_query(query) {
            return Err(McpError::InvalidParams(
                "Unsafe query detected. Only SELECT, INFO, RETURN, and LET are allowed in db.query. \
                 Use db.mutate for modifications.".into()
            ));
        }

        let full_query = format!("USE DB {}; {}", database, query);
        let rx = self.sender.query(full_query);

        // Block on the future since McpTool::call is sync
        let rt = tokio::runtime::Handle::current();
        let result = rt.block_on(async {
            rx.await.map_err(|_| McpError::Internal("DB channel closed".into()))?
                .map_err(|e| McpError::Internal(format!("Query error: {}", e)))
        })?;

        Ok(result)
    }
}

fn is_safe_query(query: &str) -> bool {
    let upper = query.trim().to_uppercase();
    upper.starts_with("SELECT")
        || upper.starts_with("INFO")
        || upper.starts_with("RETURN")
        || upper.starts_with("LET")
        || upper.starts_with("LIVE SELECT")
}
