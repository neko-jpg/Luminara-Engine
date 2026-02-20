# Asset Vault Activity Flow

This document details the frontend-to-backend transition flow for the Asset Vault activity, the central repository for project resources.

## Overview

The Asset Vault (Activity Index 5) allows managing project files, importing assets, and organizing folder structures.

## Transition Diagram

```mermaid
flowchart TD
    %% Frontend UI Layer
    subgraph UI_Layer [Frontend: GPUI View]
        AB_Icon["Activity Bar Icon (Asset Vault)"]
        AV_Box["AssetVaultBox View"]
        FileTree["Directory Tree View"]
        AssetGrid["Asset Grid (UniformList)"]
        Preview["Asset Preview Panel"]
        DropZone["Drag & Drop Zone"]
    end

    %% Service Layer
    subgraph Service_Layer [Asset Management]
        AssetDB["Asset Database (LuminaraDatabase)"]
        Importer["Asset Importer (Texture/Mesh/Audio)"]
        MetaSystem["Metadata Handler (.meta)"]
        ThumbnailGen["Thumbnail Generator"]
    end

    %% Backend System Layer
    subgraph Backend_Layer [File System / OS]
        FS_Watcher["File System Watcher (notify)"]
        OS_FS["Operating System File System"]
        IO_Thread["Async IO Thread"]
    end

    %% Activity Activation
    AB_Icon -- "on_click (Index 5)" --> AV_Box
    AV_Box -- "Render Panels" --> FileTree
    AV_Box -- "Render Panels" --> AssetGrid
    AV_Box -- "Render Panels" --> Preview

    %% Navigation Interaction
    FileTree -- "Select Folder" --> AssetDB
    AssetDB -- "Query Assets in Path" --> AssetGrid
    AssetGrid -- "Render Thumbnails" --> UI_Layer

    %% Import Interaction (Drag & Drop)
    DropZone -- "File Dropped" --> Importer
    Importer -- "Process File" --> IO_Thread
    IO_Thread -- "Write Asset" --> OS_FS
    Importer -- "Generate Metadata" --> MetaSystem
    MetaSystem -- "Write .meta" --> OS_FS

    %% Detection & Feedback
    OS_FS -- "File Created/Modified" --> FS_Watcher
    FS_Watcher -- "Event (Create)" --> AssetDB
    AssetDB -- "Index Asset" --> AssetDB
    AssetDB -- "Request Thumbnail" --> ThumbnailGen
    ThumbnailGen -- "Generate Image" --> AssetGrid
    AssetGrid -- "Update Grid View" --> UI_Layer

    %% Preview Interaction
    AssetGrid -- "Select Asset" --> Preview
    Preview -- "Load Asset Data" --> AssetDB
    AssetDB -- "Read File" --> OS_FS
    Preview -- "Render Content" --> UI_Layer
```

## Component Details

### Frontend Components
*   **AssetVaultBox:** Main container.
*   **File Tree:** Collapsible directory structure of the project assets folder.
*   **Asset Grid:** Virtualized grid displaying asset thumbnails. Supports drag-and-drop.
*   **Preview Panel:** Shows details (metadata, image preview, model viewer) of the selected asset.

### Services & Backend
*   **Asset Database:** Indexes all assets, managing UUIDs and metadata.
*   **File System Watcher:** Monitors disk changes to keep the database in sync with external modifications.
*   **Asset Importer:** Converts raw files (e.g., .png, .obj) into engine-ready formats or simply copies them.
*   **Thumbnail Generator:** Creates visual representations for assets (e.g., downscaled textures, model snapshots).
