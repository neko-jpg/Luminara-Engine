pub mod server;
pub mod tools_scene;
pub mod tools_script;
pub mod tools_viewport;

pub use server::{LuminaraMcpServer, McpTool, McpError, McpRequest, McpResponse};
pub use tools_scene::{CreateEntityTool, ModifyComponentTool, QueryEntitiesTool};
pub use tools_script::{CreateScriptTool, ModifyScriptTool, DebugInspectTool, ProjectScaffoldTool};
pub use tools_viewport::ViewportCaptureTool;

#[allow(dead_code)]
fn main() {
    println!("Luminara MCP Server library entry point for tests.");
}
