//! Luminara Editor Demo
//!
//! State-View-Command Architecture で実装されたエディタのデモ

use eframe::{NativeOptions, Renderer};
use egui::ViewportBuilder;
use luminara_core::World;

fn main() {
    // トレーシング初期化（エラーレベル調整）
    tracing_subscriber::fmt()
        .with_env_filter("warn,luminara_editor=info")
        .init();
    
    // WSL環境検出と対応
    let is_wsl = std::env::var("WSL_DISTRO_NAME").is_ok() 
        || std::env::var("WSL_INTEROP").is_ok();
    
    if is_wsl {
        println!("Running in WSL environment");
        // WSLgが利用可能かチェック
        if std::env::var("WAYLAND_DISPLAY").is_err() 
            && std::env::var("DISPLAY").is_err() {
            eprintln!("Warning: No display server detected.");
            eprintln!("Please ensure WSLg is enabled or X server is running.");
            eprintln!("Trying to run anyway...");
        }
    }
    
    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_title("Luminara Editor"),
        // ソフトウェアレンダリングフォールバックを有効化
        renderer: Renderer::default(),
        ..Default::default()
    };
    
    let result = eframe::run_native(
        "Luminara Editor",
        options,
        Box::new(|_cc| Box::new(EditorApp::new())),
    );
    
    if let Err(e) = result {
        eprintln!("Failed to run editor: {}", e);
        
        // WSLでの一般的なエラーのヒント
        if is_wsl {
            eprintln!("\n=== WSL Troubleshooting ===");
            eprintln!("1. Ensure WSLg is enabled:");
            eprintln!("   wsl --update");
            eprintln!("   wsl --shutdown");
            eprintln!("\n2. Or install X server (VcXsrv) on Windows:");
            eprintln!("   - Download from: https://sourceforge.net/projects/vcxsrv/");
            eprintln!("   - Start XLaunch with 'Disable access control'");
            eprintln!("   - Set DISPLAY=:0 in WSL");
            eprintln!("\n3. Or use software rendering:");
            eprintln!("   LIBGL_ALWAYS_SOFTWARE=1 cargo run -p editor_demo");
        }
        
        std::process::exit(1);
    }
}

struct EditorApp {
    editor: luminara_editor::LuminaraEditor,
}

impl EditorApp {
    fn new() -> Self {
        // ワールド作成
        let world = World::new();
        
        // エディタ作成
        let editor = luminara_editor::create_editor(world);
        
        Self { editor }
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // エディタ更新
        self.editor.update(ctx);
        
        // 連続更新
        ctx.request_repaint_after(std::time::Duration::from_millis(16));
    }
    
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        println!("Editor shutting down...");
    }
}
