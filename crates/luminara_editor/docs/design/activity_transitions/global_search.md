# Global Search Activity Flow

This document details the frontend-to-backend transition flow for the Global Search activity, triggered via the Activity Bar or keyboard shortcut (Cmd+K).

## Overview

The Global Search feature acts as an overlay on top of the current editor view, allowing users to quickly find assets, entities, and commands.

## Transition Diagram

```mermaid
flowchart TD
    %% Frontend UI Layer
    subgraph UI_Layer [Frontend: GPUI View]
        AB_Icon["Activity Bar Icon (Search)"]
        Kbd_Shortcut["Keyboard Shortcut (Cmd+K)"]
        EditorWindow["EditorWindow View"]
        GS_Overlay["GlobalSearch Overlay View"]
        GS_Input["Search Input Field"]
        GS_List["Result List (UniformList)"]
    end

    %% State Management Layer
    subgraph State_Layer [Shared State & Commands]
        SharedState["SharedEditorState (RwLock)"]
        Gen_Counter["Generation Counter (AtomicU64)"]
        Cmd_Toggle["Command: ToggleGlobalSearch"]
        Cmd_Nav["Command: NavigateToAsset / FocusEntity"]
    end

    %% Backend System Layer
    subgraph Backend_Layer [Backend Systems]
        EngineHandle["EngineHandle (Arc)"]
        AssetDB["Asset Database (LuminaraDatabase)"]
        ECS_World["ECS World (Entities)"]
        Query_Engine["Search Query Engine"]
    end

    %% Flows
    AB_Icon -- "on_click (Index 0)" --> EditorWindow
    Kbd_Shortcut -- "Dispatch Action" --> Cmd_Toggle
    Cmd_Toggle -- "Handle Action" --> EditorWindow

    EditorWindow -- "Call toggle_global_search()" --> SharedState
    SharedState -- "Update Visibility boolean" --> SharedState
    SharedState -- "Increment Generation" --> Gen_Counter

    %% Reactivity Loop
    Gen_Counter -. "Polled by EditorWindow Loop" .-> EditorWindow
    EditorWindow -- "Detect Change & Update View" --> GS_Overlay
    GS_Overlay -- "Check Visibility" --> GS_Overlay

    %% Search Interaction
    GS_Overlay -- "Render (if visible)" --> GS_Input
    GS_Input -- "on_input (User Types)" --> GS_Overlay
    GS_Overlay -- "Dispatch Query" --> Query_Engine

    Query_Engine -- "Query Assets" --> AssetDB
    Query_Engine -- "Query Entities" --> ECS_World

    AssetDB -- "Return Asset Matches" --> Query_Engine
    ECS_World -- "Return Entity Matches" --> Query_Engine

    Query_Engine -- "Aggregated Results" --> GS_List
    GS_List -- "Render Items" --> UI_Layer

    %% Selection Interaction
    GS_List -- "on_click (Select Item)" --> Cmd_Nav
    Cmd_Nav -- "Execute Navigation" --> EngineHandle
    EngineHandle -- "Load Asset / Focus Camera" --> ECS_World
    Cmd_Nav -- "Close Overlay" --> SharedState
```

## Component Details

### Frontend Components
*   **Activity Bar Icon:** Located at index 0. Triggers `toggle_global_search` on the `EditorWindow`.
*   **GlobalSearch View:** An overlay component (`crates/luminara_editor/src/features/global_search`) that renders the search bar and results list.
*   **Input Field:** Captures user keystrokes and debounces queries.
*   **Result List:** Uses `gpui::uniform_list` for virtualized rendering of search results.

### State & Backend
*   **SharedEditorState:** Manages the visibility state of the global search overlay across threads.
*   **LuminaraDatabase:** The backend database for asset metadata indexing.
*   **ECS World:** The active game world, queried for runtime entities matching the search term.
