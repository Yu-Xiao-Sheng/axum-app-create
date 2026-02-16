# Implementation Plan: v0.4.0 Plugin System / 实施计划：v0.4.0 插件系统

## Overview / 概述

本计划将 v0.4.0 插件系统分解为增量编码任务。每个任务在前一个任务基础上构建，确保无孤立代码。所有代码使用 Rust (edition 2024)，测试使用 proptest。插件系统新增 `src/plugin/` 模块，包含 7 个子模块。

This plan breaks down the v0.4.0 Plugin System into incremental coding tasks. Each task builds on the previous, ensuring no orphaned code. All code in Rust (edition 2024), tests use proptest. The plugin system adds a new `src/plugin/` module with 7 submodules.

## Tasks

- [x] 1. Add new dependencies and set up plugin module structure
  - [x] 1.1 Update `Cargo.toml`: bump version to `0.4.0`, add `semver = "1"`
    - _Requirements: 9.6_
  - [x] 1.2 Create `src/plugin/mod.rs` with submodule declarations (manifest, registry, loader, dependency, hooks, sandbox, manager)
    - Create empty stub files for each submodule
    - Add `pub mod plugin;` to `src/lib.rs`
    - _Requirements: 1.1_
  - [x] 1.3 Add `Plugin(String)` variant to `CliError` in `src/error.rs`
    - _Requirements: 1.5_

- [x] 2. Implement plugin manifest parser
  - [x] 2.1 Create `src/plugin/manifest.rs` with `PluginManifest`, `PluginCapabilities`, `PluginCommandDef`, `PluginPermissions` structs
    - Implement `ManifestParser::parse()`, `ManifestParser::serialize()`, `ManifestParser::validate()`, `ManifestParser::load_from_dir()`
    - Use `serde` derive for TOML serialization/deserialization
    - Validate required fields: `name`, `version`, `description`, `min_tool_version`
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8_
  - [x]* 2.2 Write property test for manifest serialization round-trip
    - **Property 1: Plugin manifest serialization round-trip**
    - Generate random PluginManifest structs, serialize to TOML then deserialize, verify equivalence
    - **Validates: Requirements 1.7, 1.8, 1.9**
  - [x]* 2.3 Write property test for required field validation
    - **Property 2: Plugin manifest required field validation**
    - Generate TOML strings missing various required fields, verify parse returns appropriate errors
    - **Validates: Requirements 1.2, 1.5**

- [x] 3. Implement plugin registry
  - [x] 3.1 Create `src/plugin/registry.rs` with `PluginEntry`, `PluginSource`, `PluginRegistry` structs
    - Implement `load()`, `save()`, `add()`, `remove()`, `find()`, `find_mut()`, `enabled_plugins()`
    - Registry file path: `~/.axum-app-create/plugins.toml`
    - _Requirements: 3.7, 3.8, 3.10_
  - [x]* 3.2 Write property test for registry round-trip and invariants
    - **Property 4: Plugin registry round-trip and invariants**
    - Generate random add/remove operation sequences, verify state consistency and serialization round-trip
    - **Validates: Requirements 3.7, 3.8, 3.10**

- [x] 4. Implement version compatibility checker
  - [x] 4.1 Add version compatibility check function using `semver` crate in `src/plugin/loader.rs`
    - Implement `PluginLoader::check_compatibility()` comparing `min_tool_version` with current tool version
    - _Requirements: 2.7, 2.8_
  - [x]* 4.2 Write property test for version compatibility
    - **Property 3: Version compatibility check**
    - Generate random semver pairs, verify compatibility logic
    - **Validates: Requirements 2.7, 2.8**

- [x] 5. Checkpoint - Ensure all tests pass
  - Ensure all tests pass (including existing tests), ask the user if questions arise.

- [ ] 6. Implement dependency resolver
  - [ ] 6.1 Create `src/plugin/dependency.rs` with `DependencyResolver` struct
    - Implement `resolve()` using Kahn's algorithm for topological sort
    - Implement `detect_cycle()` using DFS cycle detection
    - Implement `validate_dependencies()` for checking installed deps and version compatibility
    - _Requirements: 8.1, 8.3, 8.4, 8.5_
  - [ ]* 6.2 Write property test for circular dependency detection
    - **Property 11: Circular dependency detection**
    - Generate random directed graphs (with and without cycles), verify detection correctness
    - **Validates: Requirements 8.3**
  - [ ]* 6.3 Write property test for topological sort correctness
    - **Property 12: Dependency topological sort correctness**
    - Generate random DAGs, verify every plugin appears after its dependencies
    - **Validates: Requirements 8.1, 8.4**

- [ ] 7. Implement hook executor
  - [ ] 7.1 Create `src/plugin/hooks.rs` with `HookPoint`, `PluginContext`, `HookResult`, `HookExecutor` structs
    - Implement `execute()` to run hooks in priority order
    - Implement `sort_by_priority()` for ordering plugins
    - Hook errors are recorded but don't interrupt the pipeline
    - _Requirements: 4.1, 4.2, 4.3, 4.6, 4.7, 4.8, 4.9_
  - [ ]* 7.2 Write property test for hook priority ordering
    - **Property 6: Hook priority ordering**
    - Generate random priority values, verify execution order
    - **Validates: Requirements 4.2, 4.3, 4.7**
  - [ ]* 7.3 Write property test for modify_context invariant
    - **Property 7: modify_context only adds, never removes**
    - Generate random initial contexts and additions, verify originals preserved
    - **Validates: Requirements 4.4**
  - [ ]* 7.4 Write property test for error resilience
    - **Property 8: Hook errors do not interrupt pipeline**
    - Generate plugin sets with some returning errors, verify all plugins invoked
    - **Validates: Requirements 4.8**

- [ ] 8. Implement plugin sandbox
  - [ ] 8.1 Create `src/plugin/sandbox.rs` with `PluginSandbox` struct
    - Implement `validate_path()` to check paths against project dir and plugin dir whitelist
    - Implement `check_permissions()` for permission validation
    - Implement `confirm_permissions()` for interactive permission confirmation
    - _Requirements: 11.2, 11.3, 11.4, 11.5_
  - [ ]* 8.2 Write property test for sandbox path validation
    - **Property 13: Filesystem sandbox path validation**
    - Generate random paths, verify only project-dir and plugin-dir paths are allowed
    - **Validates: Requirements 11.2, 11.3**

- [ ] 9. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 10. Implement plugin loader
  - [ ] 10.1 Create `src/plugin/loader.rs` with `PluginLoader` and `LoadedPlugin` structs
    - Implement `new()` to initialize cache directory (`~/.axum-app-create/plugins/`)
    - Implement `install()` to install from PluginSource to cache
    - Implement `load_local()` to load plugin from local path
    - Implement `load_git()` to clone from Git repository using `git2`
    - Implement `load_crate()` as a stub for future crates.io support
    - Load plugin templates from `templates/` subdirectory
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.9_

- [ ] 11. Implement plugin manager (facade)
  - [ ] 11.1 Create `src/plugin/manager.rs` with `PluginManager` struct
    - Implement `new()` to initialize registry and loader
    - Implement `install()`, `uninstall()`, `enable()`, `disable()`
    - Implement `list()`, `info()`
    - Implement `load_enabled()` using DependencyResolver for topological sort
    - Implement `get_plugin_templates()` to collect templates from all loaded plugins
    - Implement `execute_hook()` delegating to HookExecutor
    - Implement `run_command()` for plugin command execution
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.8, 3.9_
  - [ ]* 11.2 Write property test for uninstall dependency detection
    - **Property 5: Uninstall dependency detection**
    - Generate dependency graphs, verify uninstalling depended-upon plugins is detected
    - **Validates: Requirements 3.9**

- [ ] 12. Implement plugin configuration merge
  - [ ] 12.1 Add plugin config parsing to `src/config/user_config.rs`
    - Extend `UserConfig` to support `[plugins.<name>]` sections
    - Implement config merge logic: manifest defaults + user overrides
    - Warn on unknown config keys
    - _Requirements: 7.1, 7.2, 7.3, 7.4_
  - [ ]* 12.2 Write property test for plugin config merge
    - **Property 10: Plugin configuration merge**
    - Generate random manifest defaults and user overrides, verify merge correctness
    - **Validates: Requirements 7.1, 7.3**

- [ ] 13. Integrate plugin templates into TemplateResolver
  - [ ] 13.1 Extend `TemplateResolver::resolve()` in `src/template/resolver.rs` to accept plugin templates
    - Add plugin templates parameter to resolve method
    - Implement three-tier merge: built-in → plugin (by priority) → user custom
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_
  - [ ]* 13.2 Write property test for template merge priority
    - **Property 9: Template merge priority**
    - Generate multi-source template sets, verify priority ordering
    - **Validates: Requirements 5.2, 5.3**

- [ ] 14. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 15. Integrate plugin system into generation flow
  - [ ] 15.1 Update `generate_project_with_templates()` in `src/generator/project.rs`
    - Initialize PluginManager and load enabled plugins
    - Execute `pre_generate` hook before template resolution
    - Execute `modify_context` hook to extend template context
    - Execute `modify_templates` hook to extend template set
    - Pass plugin templates to TemplateResolver
    - Execute `post_generate` hook after file generation
    - Handle plugin system initialization failure gracefully (continue without plugins)
    - _Requirements: 4.2, 4.3, 4.4, 4.5, 10.1, 10.4_
  - [ ]* 15.2 Write property test for backward compatibility
    - **Property 14: Backward compatibility with no plugins**
    - Generate projects with empty plugin set, compare output to no-plugin generation
    - **Validates: Requirements 10.1**

- [ ] 16. Extend CLI with plugin subcommands
  - [ ] 16.1 Add `Plugin` variant to `Commands` enum and `PluginAction` subcommand enum in `src/main.rs`
    - Add `plugin install`, `plugin uninstall`, `plugin enable`, `plugin disable`, `plugin list`, `plugin info`, `plugin run` subcommands
    - Add `--plugin <NAME>` flag to `new` subcommand
    - Update version string to `0.4.0`
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 9.6, 9.7_
  - [ ] 16.2 Wire plugin subcommands to PluginManager methods
    - Implement `run_plugin_command()` function for each plugin action
    - Display bilingual help messages
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 6.1, 6.2, 6.3_

- [ ] 17. Checkpoint - Ensure all tests pass
  - Ensure all tests pass (all existing + new tests), ask the user if questions arise.

- [ ] 18. Integration tests for plugin system
  - [ ]* 18.1 Write integration test: install local path plugin, verify registry updated
    - _Requirements: 2.1, 2.4, 3.1, 3.7_
  - [ ]* 18.2 Write integration test: install plugin with templates, generate project, verify plugin templates in output
    - _Requirements: 5.1, 5.2_
  - [ ]* 18.3 Write integration test: install plugin with hooks, generate project, verify hooks executed
    - _Requirements: 4.2, 4.3_
  - [ ]* 18.4 Write integration test: install multiple plugins with dependencies, verify load order
    - _Requirements: 8.1, 8.4_
  - [ ]* 18.5 Write integration test: disable plugin, generate project, verify plugin not active
    - _Requirements: 3.4_
  - [ ]* 18.6 Write integration test: backward compatibility — no plugins, all v0.3.0 tests pass
    - _Requirements: 10.1, 10.2, 10.3_

- [ ] 19. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes / 备注

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties (proptest, min 100 iterations)
- Unit tests validate specific examples and edge cases
- The CLI extension (task 16) is placed after all core modules are implemented to minimize integration risk
- Plugin system gracefully degrades if initialization fails (Requirement 10.4)
