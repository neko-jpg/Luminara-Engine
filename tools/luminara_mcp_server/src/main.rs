pub mod server;

pub use server::{LuminaraMcpServer, McpError, McpRequest, McpResponse, McpTool};

#[tokio::main]
async fn main() {
    println!("Luminara MCP Server starting...");
    // Main loop would be here, listening on TCP/stdio
    // For MVP task, we just implemented the struct and handler logic.
}
