# Backend & AI Activity Flow

This document details the frontend-to-backend transition flow for the Backend & AI activity, the hub for scripting and AI-assisted development.

## Overview

The Backend & AI activity (Activity Index 4) integrates code editing, terminal access, and the AI Copilot.

## Transition Diagram

```mermaid
flowchart TD
    %% Frontend UI Layer
    subgraph UI_Layer [Frontend: GPUI View]
        AB_Icon["Activity Bar Icon (Backend & AI)"]
        BAI_Box["BackendAIBox View"]
        ChatUI["AI Chat Interface (List/Input)"]
        CodeEditor["Script Editor (Monaco-like)"]
        Console["Log Console / Terminal"]
    end

    %% Service Layer
    subgraph Service_Layer [AI Service]
        AiAssistant["AiAssistant Service"]
        IntentResolver["Intent Resolver"]
        PromptEngine["Prompt Context Builder"]
        CodePipeline["Code Verification Pipeline"]
    end

    %% Backend System Layer
    subgraph Backend_Layer [MCP / LLM / Engine]
        LLM_API["LLM Provider (OpenAI/Anthropic)"]
        MCP_Server["Luminara MCP Server"]
        ECS_World["ECS World (Mutation)"]
        ScriptSys["Script System (Lua)"]
    end

    %% Activity Activation
    AB_Icon -- "on_click (Index 4)" --> BAI_Box
    BAI_Box -- "Render Panels" --> ChatUI
    BAI_Box -- "Render Panels" --> CodeEditor
    BAI_Box -- "Render Panels" --> Console

    %% User Prompt Interaction
    ChatUI -- "Submit Prompt ('Spawn Cube')" --> AiAssistant
    AiAssistant -- "Build Context (Scene State)" --> PromptEngine
    PromptEngine -- "Send Request" --> LLM_API

    LLM_API -- "Return JSON Intent / Code" --> IntentResolver

    %% Intent Resolution (Action)
    IntentResolver -- "If Code Generation" --> CodePipeline
    CodePipeline -- "Static Analysis / Dry Run" --> CodePipeline
    CodePipeline -- "Verified Script" --> CodeEditor

    IntentResolver -- "If Scene Mutation" --> MCP_Server
    MCP_Server -- "Execute Tool (CreateEntity)" --> ECS_World

    %% Feedback Loop
    ECS_World -- "Entity Created Event" --> Console
    ScriptSys -- "Script Loaded" --> Console
    Console -- "Log Output" --> ChatUI
    ChatUI -- "Update Message List" --> UI_Layer
```

## Component Details

### Frontend Components
*   **BackendAIBox:** Main container.
*   **Chat Interface:** Conversational UI for interacting with the AI Agent.
*   **Script Editor:** Syntax-highlighted text editor for manual or AI-generated code.
*   **Console:** Displays engine logs, script errors, and AI thought processes.

### Services & Backend
*   **AiAssistant:** Manages the conversation history and API communication.
*   **Intent Resolver:** Parses the LLM's response into actionable commands (e.g., `SpawnRelative`, `ModifyComponent`).
*   **MCP Server:** Exposes engine tools (Scene, FileSystem) to the AI model in a structured way.
*   **Code Verification:** Ensures generated scripts are safe and syntactically correct before execution.
