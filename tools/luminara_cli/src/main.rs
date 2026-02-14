use clap::{Parser, Subcommand};
use anyhow::Result;
use std::path::PathBuf;
use std::fs;
use luminara_cli::ops;

#[derive(Parser)]
#[command(name = "luminara")]
#[command(about = "Luminara Engine CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(subcommand)]
    Ai(AiCommands),
}

#[derive(Subcommand)]
enum AiCommands {
    Generate {
        #[arg(short, long)]
        prompt: String,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(short, long, default_value = "3d-fps")]
        template: String,
    },
    Scaffold {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        output: PathBuf,
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Ai(cmd) => match cmd {
            AiCommands::Generate { prompt, output, template } => {
                println!("Generating game from prompt: '{}' using template '{}'", prompt, template);
                generate_project(&output, &template, &prompt).await?;
            },
            AiCommands::Scaffold { name, output } => {
                println!("Scaffolding project '{}' at {:?}", name, output);
                ops::scaffold_project(&output, &name)?;
            }
        }
    }

    Ok(())
}

async fn generate_project(path: &PathBuf, _template: &str, prompt: &str) -> Result<()> {
    ops::scaffold_project(path, "generated_game")?;

    println!("Analyzing prompt with AI (Mock)...");
    let prompt_lower = prompt.to_lowercase();

    let mut entities = Vec::new();

    // Always add Main Camera
    entities.push(r#"
    (
        name: "Main Camera",
        transform: (
            translation: (0.0, 5.0, 10.0),
            rotation: (0.0, 0.0, 0.0, 1.0),
            scale: (1.0, 1.0, 1.0),
        ),
        components: [
            Camera(
                fov: 60.0,
                near: 0.1,
                far: 1000.0,
            ),
        ]
    )"#);

    // Always add Directional Light
    entities.push(r#"
    (
        name: "Sun",
        transform: (
            translation: (0.0, 10.0, 0.0),
            rotation: (0.7, 0.0, 0.0, 0.7),
            scale: (1.0, 1.0, 1.0),
        ),
        components: [
            DirectionalLight(
                color: (1.0, 1.0, 1.0),
                intensity: 1.0,
            ),
        ]
    )"#);

    if prompt_lower.contains("player") || prompt_lower.contains("character") || prompt_lower.contains("fps") || prompt_lower.contains("rpg") {
        entities.push(r#"
    (
        name: "Player",
        transform: (
            translation: (0.0, 1.0, 0.0),
            rotation: (0.0, 0.0, 0.0, 1.0),
            scale: (1.0, 1.0, 1.0),
        ),
        components: [
            Script(
                path: "assets/scripts/player.lua",
            ),
        ]
    )"#);

        let script_path = path.join("assets/scripts/player.lua");
        let script_content = r#"
local player = {}

function player.on_start()
    print("Player started")
end

function player.on_update(dt, input, world)
    -- Basic movement logic placeholder
    -- Use input:key_pressed("W") to move
end

return player
"#;
        fs::write(script_path, script_content)?;
    }

    if prompt_lower.contains("enemy") || prompt_lower.contains("monster") {
        entities.push(r#"
    (
        name: "Enemy",
        transform: (
            translation: (5.0, 1.0, 5.0),
            rotation: (0.0, 0.0, 0.0, 1.0),
            scale: (1.0, 1.0, 1.0),
        ),
        components: []
    )"#);
    }

    if prompt_lower.contains("cube") || prompt_lower.contains("box") {
         entities.push(r#"
    (
        name: "Cube",
        transform: (
            translation: (2.0, 0.5, 0.0),
            rotation: (0.0, 0.0, 0.0, 1.0),
            scale: (1.0, 1.0, 1.0),
        ),
        components: []
    )"#);
    }

    let scene_content = format!("// Generated from: {}\n(entities: [\n{}\n])", prompt, entities.join(",\n"));
    let scene_path = path.join("assets/scenes/main.ron");
    fs::write(scene_path, scene_content)?;

    println!("Project generated successfully at {:?}", path);
    Ok(())
}
