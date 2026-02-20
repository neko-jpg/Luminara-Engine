# Settings Activity Flow

This document details the frontend-to-backend transition flow for the Settings activity, managing global editor configuration.

## Overview

The Settings activity (Bottom Item 'Gear') provides a modal overlay for configuring the editor environment, keybindings, and themes.

## Transition Diagram

```mermaid
flowchart TD
    %% Frontend UI Layer
    subgraph UI_Layer [Frontend: GPUI View]
        AB_Icon["Activity Bar Icon (Settings)"]
        SettingsPanel["SettingsPanel Overlay View"]
        CategoryList["Category Sidebar (General/Editor/Theme)"]
        ContentArea["Settings Content Form"]
        InputControl["Input Control (Text/Toggle/Dropdown)"]
    end

    %% State Management Layer
    subgraph State_Layer [Configuration Model]
        SettingsStore["Settings Store (RwLock)"]
        Preferences["Preferences Struct"]
        ThemeManager["Theme Manager"]
        Keymap["Keymap Context"]
    end

    %% Backend System Layer
    subgraph Backend_Layer [Persistence & IO]
        Config_File["settings.json (Disk)"]
        File_Watcher["Config File Watcher"]
        Serializer["JSON/TOML Serializer"]
    end

    %% Activity Activation
    AB_Icon -- "on_click (Bottom Item)" --> SettingsPanel
    SettingsPanel -- "Render Categories" --> CategoryList
    CategoryList -- "Select 'Theme'" --> ContentArea
    ContentArea -- "Render Controls" --> InputControl

    %% Setting Modification
    InputControl -- "Change Value (e.g., Dark Mode)" --> SettingsStore
    SettingsStore -- "Update Preference" --> Preferences
    Preferences -- "Serialize" --> Serializer
    Serializer -- "Write to Disk" --> Config_File

    %% Hot Reload / Application of Settings
    Preferences -- "Theme Changed Event" --> ThemeManager
    ThemeManager -- "Reload Theme Assets" --> UI_Layer

    Preferences -- "Keybinding Changed Event" --> Keymap
    Keymap -- "Update Shortcuts" --> UI_Layer

    %% External Modification
    Config_File -- "File Changed" --> File_Watcher
    File_Watcher -- "Reload Preferences" --> Preferences
    Preferences -- "Notify UI" --> SettingsPanel
    SettingsPanel -- "Refresh View" --> UI_Layer
```

## Component Details

### Frontend Components
*   **SettingsPanel:** Main overlay container.
*   **Category List:** Sidebar navigation for setting groups (General, Editor, Theme, Keybindings).
*   **Content Area:** Displays the specific controls for the selected category.
*   **Input Controls:** Reusable UI widgets (Checkbox, TextInput, Dropdown) bound to setting values.

### State & Backend
*   **Settings Store:** Centralized repository for runtime configuration.
*   **Preferences Struct:** Strongly typed Rust struct representing the configuration schema.
*   **Serializer:** Handles reading/writing the configuration to disk (JSON/TOML).
*   **Theme Manager:** Applying color schemes to the UI in real-time.
