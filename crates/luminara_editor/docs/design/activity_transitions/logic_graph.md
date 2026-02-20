# Logic Graph Activity Flow

This document details the frontend-to-backend transition flow for the Logic Graph activity, used for visual scripting.

## Overview

The Logic Graph provides a node-based interface (Activity Index 2) to program game logic without writing code directly. It transpiles to Lua.

## Transition Diagram

```mermaid
flowchart TD
    %% Frontend UI Layer
    subgraph UI_Layer [Frontend: GPUI View]
        AB_Icon["Activity Bar Icon (Logic Graph)"]
        LG_Box["LogicGraphBox View"]
        Palette["Node Palette"]
        Canvas["Graph Canvas (SVG/Canvas)"]
        Inspector["Node Inspector"]
    end

    %% State Management Layer
    subgraph State_Layer [Graph Model]
        GraphState["Graph Data Structure (Nodes/Edges)"]
        SelectionState["Selected Node ID"]
        CmdQueue["Command Queue (AddNode/Connect)"]
    end

    %% Backend System Layer
    subgraph Backend_Layer [Script Compilation]
        Transpiler["Lua Transpiler"]
        ScriptMgr["Script Manager"]
        LuaVM["Lua Runtime (mlua)"]
        HotReload["Hot Reload System"]
    end

    %% Activity Activation
    AB_Icon -- "on_click (Index 2)" --> LG_Box
    LG_Box -- "Render" --> Palette
    LG_Box -- "Render" --> Canvas

    %% Graph Editing
    Palette -- "Drag Node" --> Canvas
    Canvas -- "Drop Node" --> CmdQueue
    Canvas -- "Connect Pins" --> CmdQueue

    CmdQueue -- "Mutate Graph" --> GraphState
    GraphState -- "Graph Changed Event" --> Canvas
    Canvas -- "Re-render Nodes/Lines" --> UI_Layer

    %% Selection
    Canvas -- "Click Node" --> SelectionState
    SelectionState -- "Update Inspector" --> Inspector
    Inspector -- "Modify Property" --> CmdQueue

    %% Compilation & Runtime
    GraphState -- "On Save / Auto-Compile" --> Transpiler
    Transpiler -- "Generate Lua Source" --> ScriptMgr
    ScriptMgr -- "Write to Disk / Mem" --> HotReload

    HotReload -- "Detect Change" --> LuaVM
    LuaVM -- "Load Chunk" --> LuaVM
    LuaVM -- "Execute Logic" --> ECS_World
```

## Component Details

### Frontend Components
*   **LogicGraphBox:** Container for the node editor interface.
*   **Canvas:** Renders nodes and connections. Handles mouse events for panning, zooming, and wiring.
*   **Palette:** List of available logic nodes (Events, Actions, Variables).

### State & Backend
*   **Graph Data Structure:** An in-memory representation of the visual graph (Nodes, Pins, Edges).
*   **Lua Transpiler:** Converts the graph structure into valid Lua code (e.g., `function on_update(dt) ... end`).
*   **Hot Reload System:** Monitors the generated script files and reloads the Lua VM state without restarting the editor.
