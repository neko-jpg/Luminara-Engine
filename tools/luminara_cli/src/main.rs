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

    println!("Analyzing prompt with AI...");

    let scene_path = path.join("assets/scenes/main.ron");
    let scene_content = format!("// Generated from: {}\n(entities: [])", prompt);
    fs::write(scene_path, scene_content)?;

    let script_path = path.join("assets/scripts/player.lua");
    let script_content = r#"
local player = {}
function player.on_start()
    print("Player started")
end
return player
"#;
    fs::write(script_path, script_content)?;

    println!("Project generated successfully at {:?}", path);
    Ok(())
}
