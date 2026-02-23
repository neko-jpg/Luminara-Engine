//! Theme system for Luminara Editor (Vizia version)

use vizia::prelude::*;

pub const STYLE: &str = r#"
    /* Global Resets */
    * {
        font-family: "Segoe UI", "Inter", sans-serif;
        color: #e0e0e0;
        border-color: #3a3a3a;
    }

    /* Layout Containers */
    .sidebar-left {
        width: 260px;
        background-color: #2a2a2a;
        border-right-width: 1px;
        border-color: #3e3e3e;
    }

    .sidebar-right {
        width: 320px;
        background-color: #2a2a2a;
        border-left-width: 1px;
        border-color: #3e3e3e;
    }

    .center-panel {
        background-color: #1e1e1e;
        child-space: 1s;
    }

    /* Headers & Toolbars */
    .panel-header {
        height: 32px;
        background-color: #323232;
        border-bottom-width: 1px;
        border-color: #404040;
        padding-left: 12px;
        padding-right: 8px;
        display: flex;
        flex-direction: row;
        align-items: center;
    }

    .panel-header-text {
        font-size: 11px;
        font-weight: bold;
        color: #bbbbbb;
    }

    .hierarchy-toolbar {
        height: 36px;
        padding: 4px;
        col-between: 4px;
        background-color: #2a2a2a;
    }

    .viewport-toolbar {
        height: 32px;
        background-color: #282828cc; /* Semi-transparent */
        padding-left: 12px;
        padding-right: 12px;
        border-bottom-width: 1px;
        border-color: #4a4a4a;
        display: flex;
        flex-direction: row;
        align-items: center;
    }

    .toolbar-group {
        display: flex;
        flex-direction: row;
        col-between: 2px;
    }

    .toolbar-label {
        font-size: 11px;
        color: #aaaaaa;
        top: 1s;
        bottom: 1s;
    }

    /* Activity Bar */
    .activity-bar {
        width: 52px;
        background-color: #2c2c2c;
        border-right-width: 1px;
        border-color: #3e3e3e;
        padding-top: 12px;
        padding-bottom: 12px;
        row-between: 8px;
        align-items: center;
    }

    .activity-item {
        width: 48px;
        height: 48px;
        border-radius: 6px;
        display: flex;
        justify-content: center;
        align-items: center;
    }

    .activity-item:hover {
        background-color: #3a3a3a;
    }

    .activity-item.active {
        background-color: #3a5f8a;
    }

    .activity-item.active .icon {
        color: white;
    }

    .activity-item .icon {
        color: #aaaaaa;
    }

    .activity-item.is-folder {
        background-color: #3a4a6a;
        border-width: 1px;
        border-color: #8a8aff;
    }

    .folder-badge {
        background-color: #4a6a9a;
        color: white;
        font-size: 9px;
        font-weight: bold;
        border-radius: 10px;
        padding-left: 4px;
        padding-right: 4px;
    }

    /* Icons */
    .icon {
        width: 24px;
        height: 24px;
    }

    .icon-small {
        width: 16px;
        height: 16px;
    }

    .icon-tiny {
        width: 12px;
        height: 12px;
    }

    .icon-medium {
        width: 32px;
        height: 32px;
    }

    .icon-large {
        width: 64px;
        height: 64px;
    }

    .accent-icon {
        color: #8a8aff;
    }

    .success-icon {
        color: #55ff55;
    }

    .warning-icon {
        color: #ffaa55;
    }

    .text-muted {
        color: #6a6a6a;
    }

    /* Buttons & Inputs */
    .icon-btn {
        width: 28px;
        height: 28px;
        border-radius: 4px;
        background-color: transparent;
        display: flex;
        justify-content: center;
        align-items: center;
    }

    .icon-btn:hover {
        background-color: #3f3f3f;
    }

    .tool-btn {
        height: 26px;
        padding-left: 8px;
        padding-right: 8px;
        background-color: #3a3a3a;
        border-radius: 4px;
        display: flex;
        flex-direction: row;
        align-items: center;
        col-between: 4px;
    }

    .tool-btn:hover {
        background-color: #4a4a4a;
    }

    .tool-btn.active {
        background-color: #3a5f8a;
        color: white;
    }

    .search-box {
        height: 28px;
        background-color: #3a3a3a;
        border-radius: 4px;
        border-width: 1px;
        border-color: #4a4a4a;
        color: #eeeeee;
        padding-left: 8px;
    }

    /* Tree View */
    .tree-view {
        padding-top: 4px;
    }

    .tree-item {
        height: 24px;
        padding-left: 4px;
        display: flex;
        flex-direction: row;
        align-items: center;
        col-between: 6px;
    }

    .tree-item:hover {
        background-color: #3f3f3f;
    }

    .tree-item.selected {
        background-color: #2d5a88;
        border-left-width: 2px;
        border-color: #8a8aff;
    }

    .tree-item-text {
        font-size: 12px;
        color: #dddddd;
    }

    /* Inspector */
    .inspector-content {
        padding: 12px;
        row-between: 8px;
    }

    .inspector-row {
        height: 24px;
        display: flex;
        flex-direction: row;
        align-items: center;
        col-between: 8px;
    }

    .component-box {
        background-color: #2d2d2d;
        border-radius: 4px;
        border-width: 1px;
        border-color: #3e3e3e;
        padding-bottom: 8px;
    }

    .component-header {
        height: 28px;
        background-color: #323232;
        padding-left: 8px;
        padding-right: 8px;
        border-bottom-width: 1px;
        border-color: #3e3e3e;
        display: flex;
        flex-direction: row;
        align-items: center;
    }

    .component-title {
        font-weight: bold;
        font-size: 12px;
    }

    .prop-row {
        padding-left: 8px;
        padding-right: 8px;
        padding-top: 4px;
        display: flex;
        flex-direction: row;
        align-items: center;
    }

    .prop-label {
        font-size: 11px;
        color: #aaaaaa;
    }

    .vector-field {
        display: flex;
        flex-direction: row;
        align-items: center;
        width: 1s;
        col-between: 2px;
    }

    .prop-input {
        height: 20px;
        background-color: #1e1e1e;
        border-width: 1px;
        border-color: #3e3e3e;
        border-radius: 3px;
        font-size: 11px;
        padding-left: 4px;
        width: 1s;
    }

    .axis-label-x { font-size: 10px; color: #ff5555; }
    .axis-label-y { font-size: 10px; color: #55ff55; }
    .axis-label-z { font-size: 10px; color: #5555ff; }

    .btn-primary {
        height: 28px;
        background-color: #3a5f8a;
        border-radius: 4px;
        display: flex;
        justify-content: center;
        align-items: center;
    }

    .btn-primary:hover {
        background-color: #4a7ab0;
    }

    /* Global Search */
    .global-search-container {
        background-color: #1a1a1a;
        child-space: 1s;
    }

    .search-left-panel {
        width: 40%;
        background-color: #1e1e1e;
        border-right-width: 1px;
        border-color: #2a2a2a;
    }

    .search-right-panel {
        width: 60%;
        background-color: #202020;
    }

    .search-bar-large {
        height: 48px;
        background-color: #2a2a2a;
        border-radius: 24px;
        border-width: 1px;
        border-color: #3a3a3a;
        display: flex;
        flex-direction: row;
        align-items: center;
        padding-left: 16px;
        padding-right: 16px;
        margin: 12px;
    }

    .search-input-large {
        font-size: 18px;
        background-color: transparent;
        border-width: 0px;
    }

    .filter-bar {
        padding-left: 12px;
        padding-bottom: 8px;
        display: flex;
        flex-direction: row;
        col-between: 8px;
    }

    .filter-chip {
        height: 24px;
        border-radius: 12px;
        background-color: #2a2a2a;
        border-width: 1px;
        border-color: #3a3a3a;
        padding-left: 10px;
        padding-right: 10px;
        display: flex;
        flex-direction: row;
        align-items: center;
        col-between: 4px;
    }

    .filter-chip.active {
        background-color: #3a5f8a;
        border-color: #8a8aff;
    }

    .result-item {
        height: 48px;
        padding-left: 12px;
        padding-right: 12px;
        display: flex;
        flex-direction: row;
        align-items: center;
        col-between: 12px;
        margin-left: 8px;
        margin-right: 8px;
        border-radius: 6px;
    }

    .result-item:hover {
        background-color: #2d5a88;
    }

    .result-item.selected {
        background-color: #2d5a88;
        border-width: 1px;
        border-color: #8a8aff;
    }

    .result-title { font-size: 14px; font-weight: bold; }
    .result-subtitle { font-size: 11px; color: #888888; }

    .type-badge {
        font-size: 9px;
        background-color: #3a3a3a;
        border-radius: 8px;
        padding-left: 6px;
        padding-right: 6px;
        color: #aaaaaa;
    }

    .preview-header {
        height: 60px;
        padding: 20px;
        border-bottom-width: 1px;
        border-color: #2a2a2a;
        display: flex;
        flex-direction: row;
        align-items: center;
        col-between: 12px;
    }

    .h2 { font-size: 16px; font-weight: bold; }

    .preview-box {
        height: 200px;
        background-color: #2a2a2a;
        border-radius: 12px;
        border-width: 1px;
        border-color: #3a3a3a;
        display: flex;
        justify-content: center;
        align-items: center;
        col-between: 12px;
        margin: 20px;
    }

    .meta-info {
        padding-left: 20px;
        padding-right: 20px;
        row-between: 12px;
    }

    .meta-item {
        background-color: #2a2a2a;
        border-radius: 12px;
        padding: 16px;
        border-width: 1px;
        border-color: #3a3a3a;
    }

    .meta-label { font-size: 11px; color: #888888; width: 100px; }
    .meta-value { font-size: 14px; font-weight: 500; }

    .code-block {
        background-color: #1e1e1e;
        border-radius: 8px;
        border-width: 1px;
        border-color: #4a4a4a;
        padding: 12px;
        margin: 20px;
    }

    .code-text { font-family: monospace; font-size: 12px; color: #cccccc; }

    /* Director */
    .timeline-panel {
        background-color: #1a1a1a;
        height: 200px;
        border-top-width: 1px;
        border-color: #3e3e3e;
    }

    .transport-btn {
        width: 32px;
        height: 24px;
        background-color: #3a3a3a;
        border-radius: 4px;
        display: flex;
        justify-content: center;
        align-items: center;
    }

    .transport-btn-primary {
        width: 32px;
        height: 24px;
        background-color: #3a5f8a;
        border-radius: 4px;
        display: flex;
        justify-content: center;
        align-items: center;
    }

    /* Logic Graph */
    .graph-canvas {
        background-color: #1e1e1e;
        child-space: 1s;
    }

    .graph-node {
        width: 150px;
        background-color: #2d2d2d;
        border-width: 1px;
        border-color: #4a4a4a;
        border-radius: 6px;
        position-type: self-directed;
        padding-bottom: 8px;
    }

    .node-header {
        height: 24px;
        background-color: #3a3a3a;
        color: white;
        font-size: 11px;
        font-weight: bold;
        padding-left: 8px;
        display: flex;
        align-items: center;
    }

    .node-port-in { font-size: 10px; color: #ffaa8a; padding-left: 4px; }
    .node-port-out { font-size: 10px; color: #8a8aff; padding-right: 4px; text-align: right; }

    /* Backend & AI */
    .file-item {
        height: 22px;
        padding-left: 8px;
        display: flex;
        flex-direction: row;
        align-items: center;
        col-between: 6px;
    }

    .file-item.selected {
        background-color: #2d5a88;
    }

    .chat-msg-user {
        background-color: #3a3a3a;
        padding: 8px;
        border-radius: 8px;
        margin: 8px;
        align-self: flex-end;
        color: white;
    }

    .chat-msg-ai {
        background-color: #2a2a3a;
        padding: 8px;
        border-radius: 8px;
        margin: 8px;
        align-self: flex-start;
        border-width: 1px;
        border-color: #8a8aff;
    }
"#;
