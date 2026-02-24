// Plugin system for axum-app-create
//
// This module provides the plugin architecture for extending
// the CLI tool with custom templates, hooks, commands, and more.

pub mod dependency;
pub mod hooks;
pub mod loader;
pub mod manager;
pub mod manifest;
pub mod registry;
pub mod sandbox;
