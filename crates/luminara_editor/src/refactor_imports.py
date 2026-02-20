import os
import re

MAPPINGS = {
    r'crate::app\b': r'crate::core::app',
    r'crate::window\b': r'crate::core::window',
    r'crate::preferences\b': r'crate::core::preferences',
    r'crate::commands\b': r'crate::core::commands',
    r'crate::input\b': r'crate::core::input',
    r'crate::settings\b': r'crate::core::settings',
    r'crate::viewport\b': r'crate::core::viewport',
    r'crate::engine\b': r'crate::services::engine_bridge',
    r'crate::backend_ai\b': r'crate::services::ai_agent',
    r'crate::asset_source\b': r'crate::services::asset_server',
    r'crate::theme\b': r'crate::ui::theme',
    r'crate::activity_bar\b': r'crate::ui::layouts::activity_bar',
    r'crate::resizable_panel\b': r'crate::ui::layouts::resizable_panel',
    r'crate::director\b': r'crate::features::director',
    r'crate::logic_graph\b': r'crate::features::logic_graph',
    r'crate::assetvault\b': r'crate::features::asset_vault',
    r'crate::extension\b': r'crate::features::extension',
    r'crate::account\b': r'crate::features::account',
    r'crate::scenebuilder\b': r'crate::features::scene_builder',
    r'crate::global_search\b': r'crate::features::global_search',
    # Also fix some super:: imports if applicable, but crate:: is what we mostly use
}

def process_file(filepath):
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    original = content
    for pattern, replacement in MAPPINGS.items():
        content = re.sub(pattern, replacement, content)

    if content != original:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"Updated {filepath}")

def main():
    src_dir = r"c:\dev\Luminara-Engine\crates\luminara_editor\src"
    
    for root, dirs, files in os.walk(src_dir):
        for file in files:
            if file.endswith('.rs'):
                # skip lib.rs itself as we already handled it manually
                if file == 'lib.rs' and root == src_dir:
                    continue
                process_file(os.path.join(root, file))

if __name__ == "__main__":
    main()
