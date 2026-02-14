pub mod ops {
    use std::path::PathBuf;
    use std::fs;
    use anyhow::Result;

    pub fn scaffold_project(path: &PathBuf, name: &str) -> Result<()> {
        fs::create_dir_all(path.join("assets/scenes"))?;
        fs::create_dir_all(path.join("assets/scripts"))?;
        fs::create_dir_all(path.join("src"))?;

        let cargo_toml = format!(r#"
[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
luminara = {{ path = "../../crates/luminara" }}
"#, name);

        fs::write(path.join("Cargo.toml"), cargo_toml)?;

        let main_rs = r#"
use luminara::prelude::*;

fn main() {
    App::new().run();
}
"#;
        fs::write(path.join("src/main.rs"), main_rs)?;

        Ok(())
    }
}
