//! Example: Migrate RON scene files to database
//!
//! This example demonstrates how to use the RON migration tool to convert
//! existing scene files to the database format.
//!
//! Usage:
//! ```bash
//! cargo run --example migrate_ron_scene -- assets/scenes/phase1_demo.scene.ron
//! ```

use luminara_db::{LuminaraDatabase, RonMigrationTool};
use std::env;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Get command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <scene_file.ron> [output_db_path]", args[0]);
        eprintln!("\nExample:");
        eprintln!("  {} assets/scenes/phase1_demo.scene.ron", args[0]);
        eprintln!("  {} assets/scenes/*.scene.ron data/migrated.db", args[0]);
        std::process::exit(1);
    }

    let scene_path = &args[1];
    let db_path = if args.len() > 2 {
        &args[2]
    } else {
        "data/migrated.db"
    };

    println!("=== RON Scene Migration Tool ===\n");
    println!("Scene file: {}", scene_path);
    println!("Database: {}\n", db_path);

    // Initialize database
    println!("Initializing database...");
    let db = LuminaraDatabase::new_memory().await?;
    println!("✓ Database initialized\n");

    // Create migration tool
    let tool = RonMigrationTool::new(db.clone());

    // Migrate scene
    println!("Migrating scene...");
    let start = std::time::Instant::now();

    let stats = tool.migrate_file(scene_path).await?;

    let duration = start.elapsed();

    println!("✓ Migration complete!\n");

    // Print statistics
    println!("=== Migration Statistics ===");
    println!("Entities migrated:       {}", stats.entities_migrated);
    println!("Components migrated:     {}", stats.components_migrated);
    println!("Relationships preserved: {}", stats.relationships_preserved);
    println!("Duration:                {:?}", duration);
    println!();

    // Verify migration
    println!("Verifying migration...");
    
    // Read and parse the scene file again for verification
    let content = std::fs::read_to_string(scene_path)?;
    let scene: luminara_db::migration::Scene = ron::from_str(&content)?;
    
    let is_valid = tool.verify_migration(&scene).await?;

    if is_valid {
        println!("✓ Migration verified successfully!\n");
    } else {
        println!("✗ Migration verification failed!\n");
        std::process::exit(1);
    }

    // Show database statistics
    println!("=== Database Statistics ===");
    let db_stats = db.get_statistics().await?;
    println!("Total entities:   {}", db_stats.entity_count);
    println!("Total components: {}", db_stats.component_count);
    println!("Total assets:     {}", db_stats.asset_count);
    println!("Total operations: {}", db_stats.operation_count);
    println!();

    // Query some entities
    println!("=== Sample Entities ===");
    let entities = db.query_entities("SELECT * FROM entity LIMIT 5").await?;
    for entity in entities {
        println!("- {} (tags: {:?})", 
            entity.name.unwrap_or_else(|| "Unnamed".to_string()),
            entity.tags
        );
    }
    println!();

    println!("Migration completed successfully!");
    println!("\nNote: This example uses in-memory database.");
    println!("For persistent storage, use RocksDB backend in production.");

    Ok(())
}
