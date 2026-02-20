# Director Activity Flow

This document details the frontend-to-backend transition flow for the Director activity, used for creating cinematic sequences and animations.

## Overview

The Director activity (Activity Index 3) provides a timeline interface for animating entity properties over time.

## Transition Diagram

```mermaid
flowchart TD
    %% Frontend UI Layer
    subgraph UI_Layer [Frontend: GPUI View]
        AB_Icon["Activity Bar Icon (Director)"]
        DirectorBox["DirectorBox View"]
        Timeline["Timeline View (Tracks/Keys)"]
        Transport["Transport Controls (Play/Pause)"]
        KeyframeBtn["Add Keyframe Button"]
        Scrubber["Timeline Scrubber"]
    end

    %% State Management Layer
    subgraph State_Layer [Animation Context]
        PlayState["Playback State (Playing/Paused)"]
        CurrentTime["Current Time (float)"]
        SelectedTrack["Selected Track/Entity"]
        KeyframeData["Keyframe Data (Map<Time, Value>)"]
    end

    %% Backend System Layer
    subgraph Backend_Layer [Engine Animation System]
        AnimSystem["Animation System (SystemParam)"]
        Interpolator["Curve Interpolator (Lerp/Slerp)"]
        ECS_World["ECS World (Entities)"]
        PropertySystem["Property Reflection System"]
    end

    %% Activity Activation
    AB_Icon -- "on_click (Index 3)" --> DirectorBox
    DirectorBox -- "Render Timeline" --> Timeline
    DirectorBox -- "Render Controls" --> Transport

    %% Playback Interaction
    Transport -- "Click Play" --> PlayState
    PlayState -- "Update Frame" --> AnimSystem
    AnimSystem -- "Advance Time" --> CurrentTime
    CurrentTime -- "Sync Scrubber" --> Scrubber

    %% Scrubbing Interaction
    Scrubber -- "Drag Handle" --> CurrentTime
    CurrentTime -- "Update Animation State" --> AnimSystem

    %% Keyframing Interaction
    KeyframeBtn -- "Click (Record)" --> AnimSystem
    AnimSystem -- "Capture Current Value" --> PropertySystem
    PropertySystem -- "Read Component" --> ECS_World
    PropertySystem -- "Store Keyframe" --> KeyframeData
    KeyframeData -- "Update UI" --> Timeline

    %% Evaluation Loop
    AnimSystem -- "Evaluate Curves at Time T" --> Interpolator
    Interpolator -- "Calculate Value" --> PropertySystem
    PropertySystem -- "Apply Value to Component" --> ECS_World
    ECS_World -- "Render Scene" --> DirectorBox
```

## Component Details

### Frontend Components
*   **DirectorBox:** Main container.
*   **Timeline:** Visualizes animation tracks and keyframes. Supports zooming and panning.
*   **Transport Controls:** Standard media controls (Play, Pause, Stop, Loop).
*   **Scrubber:** Interactive element to seek to a specific time.

### State & Backend
*   **Animation System:** Manages the playback loop and time synchronization.
*   **Keyframe Data:** Stores time-value pairs for properties (Position, Rotation, Opacity, etc.).
*   **Curve Interpolator:** Calculates intermediate values between keyframes (Linear, Bezier, etc.).
*   **Property Reflection:** Allows the animation system to read/write arbitrary component fields dynamically.
