# Extensions Activity Flow

This document details the frontend-to-backend transition flow for the Extensions activity, managing editor plugins and add-ons.

## Overview

The Extensions activity (Activity Index 6) allows users to browse, install, and manage plugins from the Luminara Marketplace.

## Transition Diagram

```mermaid
flowchart TD
    %% Frontend UI Layer
    subgraph UI_Layer [Frontend: GPUI View]
        AB_Icon["Activity Bar Icon (Extensions)"]
        Ext_Box["ExtensionBox View"]
        Marketplace["Marketplace Tab"]
        Installed["Installed Tab"]
        Search["Search Input"]
        InstallBtn["Install Button"]
        DetailPanel["Detail Panel"]
    end

    %% Service Layer
    subgraph Service_Layer [Extension Management]
        PluginMgr["Plugin Manager (Service)"]
        RegistryClient["Marketplace Registry Client"]
        DependencyResolver["Dependency Resolver"]
        Loader["Dynamic Loader (libloading / Lua)"]
    end

    %% Backend System Layer
    subgraph Backend_Layer [Network & IO]
        HTTP_Client["HTTP Client (reqwest)"]
        File_IO["File System (Unzip / Write)"]
        Plugin_Dir["plugins/ Directory"]
        App_Runtime["Application Runtime (App)"]
    end

    %% Activity Activation
    AB_Icon -- "on_click (Index 6)" --> Ext_Box
    Ext_Box -- "Render Tabs" --> Marketplace
    Ext_Box -- "Render Tabs" --> Installed

    %% Marketplace Browsing
    Marketplace -- "Query Extensions" --> RegistryClient
    RegistryClient -- "Fetch JSON Metadata" --> HTTP_Client
    HTTP_Client -- "Return List" --> RegistryClient
    RegistryClient -- "Populate List" --> Marketplace
    Marketplace -- "Click Item" --> DetailPanel

    %% Install Flow
    DetailPanel -- "Click Install" --> PluginMgr
    PluginMgr -- "Resolve Dependencies" --> DependencyResolver
    PluginMgr -- "Request Download" --> HTTP_Client
    HTTP_Client -- "Stream Binary/Zip" --> File_IO
    File_IO -- "Extract" --> Plugin_Dir

    %% Loading Flow
    Plugin_Dir -- "File Written" --> PluginMgr
    PluginMgr -- "Load Plugin" --> Loader
    Loader -- "Register Plugins" --> App_Runtime
    App_Runtime -- "Update System State" --> Ext_Box

    %% Feedback
    PluginMgr -- "Install Complete Event" --> DetailPanel
    DetailPanel -- "Update Button (Installed)" --> UI_Layer
    Ext_Box -- "Notify User" --> UI_Layer
```

## Component Details

### Frontend Components
*   **ExtensionBox:** Main container.
*   **Marketplace Tab:** Lists available extensions from the remote registry.
*   **Installed Tab:** Lists locally installed extensions.
*   **Detail Panel:** Shows description, version, and dependencies of a selected extension.

### Services & Backend
*   **Plugin Manager:** Core service for handling the lifecycle (Install, Enable, Disable, Uninstall) of extensions.
*   **Registry Client:** Communicates with the online extension marketplace API.
*   **Dynamic Loader:** Responsible for safely loading compiled plugins (`.dll`/`.so`) or script packages (`.lua`) into the running editor.
