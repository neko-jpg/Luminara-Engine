# Account Activity Flow

This document details the frontend-to-backend transition flow for the Account activity, managing user authentication and cloud settings.

## Overview

The Account activity (Bottom Item 'User') provides a modal for login, registration, and profile management.

## Transition Diagram

```mermaid
flowchart TD
    %% Frontend UI Layer
    subgraph UI_Layer [Frontend: GPUI View]
        AB_Icon["Activity Bar Icon (User)"]
        AccountPanel["AccountPanel Overlay View"]
        LoginForm["Login Form (Email/Pass)"]
        ProfileView["Profile Info (Avatar/Plan)"]
        LogoutBtn["Logout Button"]
    end

    %% State Management Layer
    subgraph State_Layer [Session Management]
        AuthStore["Auth Store (Token/User Info)"]
        SharedState["SharedEditorState"]
        Cmd_Login["Command: Login"]
        Cmd_Logout["Command: Logout"]
    end

    %% Backend System Layer
    subgraph Backend_Layer [Auth & Persistence]
        AuthService["Auth Service (HTTP)"]
        Keyring["OS Keyring / Secure Store"]
        CloudConfig["Cloud Sync Service"]
    end

    %% Activity Activation
    AB_Icon -- "on_click (Bottom Item)" --> SharedState
    SharedState -- "Toggle Visibility" --> AccountPanel

    %% View Logic (Conditional Rendering)
    AuthStore -- "Is Authenticated?" --> AccountPanel
    AccountPanel -- "No (Show Login)" --> LoginForm
    AccountPanel -- "Yes (Show Profile)" --> ProfileView

    %% Login Flow
    LoginForm -- "Submit Credentials" --> Cmd_Login
    Cmd_Login -- "Dispatch Request" --> AuthService
    AuthService -- "POST /api/login" --> Backend_API["External Auth Server"]
    Backend_API -- "Return JWT Token" --> AuthService
    AuthService -- "Store Token" --> Keyring
    AuthService -- "Update Session" --> AuthStore

    %% Feedback Loop
    AuthStore -- "User Logged In Event" --> AccountPanel
    AccountPanel -- "Switch View to Profile" --> UI_Layer
    AuthStore -- "Fetch Cloud Settings" --> CloudConfig
    CloudConfig -- "Sync Preferences" --> SharedState

    %% Logout Flow
    LogoutBtn -- "Click" --> Cmd_Logout
    Cmd_Logout -- "Clear Session" --> AuthService
    AuthService -- "Delete Token" --> Keyring
    AuthService -- "Update Session" --> AuthStore
    AuthStore -- "User Logged Out Event" --> AccountPanel
    AccountPanel -- "Switch View to Login" --> UI_Layer
```

## Component Details

### Frontend Components
*   **AccountPanel:** Overlay container.
*   **Login Form:** Inputs for email/password or OAuth buttons.
*   **Profile View:** Displays user avatar, name, and subscription tier (Free/Pro).

### State & Backend
*   **Auth Store:** Holds the current user's session state in memory.
*   **Auth Service:** Handles communication with the Luminara backend (e.g., Supabase/Firebase/Custom).
*   **Keyring:** Securely stores the refresh token on the user's OS keychain to persist login across sessions.
*   **Cloud Sync:** Synchronizes editor settings (theme, keybindings) to the cloud upon login.
