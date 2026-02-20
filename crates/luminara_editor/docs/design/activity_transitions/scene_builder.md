# Scene Builder Activity Flow

This document details the frontend-to-backend transition flow for the Scene Builder activity, the primary interface for world construction.

## Overview

The Scene Builder is the default view (Activity Index 1) and provides a 3D viewport, hierarchy tree, and component inspector.

## Transition Diagram

```mermaid
flowchart TD
    %% Frontend UI Layer
    subgraph UI_Layer [Frontend: GPUI View]
        AB_Icon["Activity Bar Icon (Scene Builder)"]
        EditorWindow["EditorWindow View"]
        SB_Box["SceneBuilderBox View"]
        Hierarchy["Hierarchy Panel (Tree View)"]
        Inspector["Inspector Panel (Form)"]
        Viewport["3D Viewport (WGPU Surface)"]
        Gizmo["Transform Gizmo (Interactive)"]
    end

    %% Command / Bridge Layer
    subgraph Bridge_Layer [Engine Bridge]
        EngineHandle["EngineHandle (Arc)"]
        CmdQueue["Command Queue"]
        InputSystem["Input System (Mouse/Key)"]
        SelectionState["Selection State (Shared)"]
    end

    %% Backend System Layer
    subgraph Backend_Layer [Backend: ECS World]
        World["ECS World (Components)"]
        RenderSystem["Render System (WGPU)"]
        TransformSystem["Transform System"]
        ScriptSystem["Script Runtime (Lua)"]
    end

    %% Activity Activation
    AB_Icon -- "on_click (Index 1)" --> EditorWindow
    EditorWindow -- "Set Active Index (1)" --> EditorWindow
    EditorWindow -- "Render SceneBuilderBox" --> SB_Box

    SB_Box -- "Render Panels" --> Hierarchy
    SB_Box -- "Render Panels" --> Inspector
    SB_Box -- "Render Viewport" --> Viewport

    %% Hierarchy Interaction
    Hierarchy -- "Select Entity" --> SelectionState
    SelectionState -- "Update Inspector" --> Inspector
    SelectionState -- "Update Gizmo" --> Gizmo

    Hierarchy -- "Add Entity Button" --> CmdQueue
    CmdQueue -- "Dispatch CreateEntity" --> EngineHandle
    EngineHandle -- "Apply Command" --> World

    %% Viewport Interaction
    Viewport -- "Mouse Move/Drag" --> InputSystem
    InputSystem -- "Raycast Selection" --> SelectionState
    InputSystem -- "Gizmo Drag" --> TransformSystem

    TransformSystem -- "Mutate Transform Component" --> World

    %% Inspector Interaction
    Inspector -- "Modify Field (e.g., Position)" --> CmdQueue
    CmdQueue -- "Dispatch ModifyComponent" --> EngineHandle
    EngineHandle -- "Apply Modification" --> World

    %% Feedback Loop
    World -- "Component Changed Event" --> Hierarchy
    World -- "Transform Changed" --> RenderSystem
    RenderSystem -- "Draw Frame" --> Viewport
    ScriptSystem -- "OnUpdate()" --> World
```

## Component Details

### Frontend Components
*   **SceneBuilderBox:** Main container layout.
*   **Hierarchy:** Displays the entity tree using `gpui::list`. It listens for World changes to update the tree structure.
*   **Inspector:** Dynamically generates UI forms based on selected entity components using reflection.
*   **Viewport:** Renders the 3D scene using `wgpu` integration within a GPUI element.

### Bridge & Backend
*   **EngineHandle:** Provides thread-safe access to the engine's command queue.
*   **SelectionState:** Tracks the currently selected entity ID(s) to synchronize the Hierarchy, Inspector, and Gizmo.
*   **Command Queue:** Buffers mutations (Create, Delete, Modify) to be applied at the start of the next frame to ensure thread safety.
