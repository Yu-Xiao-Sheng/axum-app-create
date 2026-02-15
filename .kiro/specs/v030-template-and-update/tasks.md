# Implementation Plan: v0.3.0 Template and Update / 实施计划：v0.3.0 模板与更新

## Overview / 概述

本计划将 v0.3.0 的三大功能（自定义模板系统、模板继承、项目更新机制）分解为增量编码任务。每个任务在前一个任务基础上构建，确保无孤立代码。所有代码使用 Rust (edition 2024)，测试使用 proptest。

This plan breaks down the three v0.3.0 features (Custom Template System, Template Inheritance, Project Update Mechanism) into incremental coding tasks. Each task builds on the previous, ensuring no orphaned code. All code in Rust (edition 2024), tests use proptest.

## Tasks

- [x] 1. Add new dependencies and update version
  - [x] 1.1 Update `Cargo.toml`: bump version to `0.3.0`, add `sha2 = "0.10"`, `toml = "0.8"`, `similar = "2"`, `dirs = "6"`
    - _Requirements: 8.6_
  - [x] 1.2 Update version string in `src/main.rs` CLI command attribute to `0.3.0`
    - _Requirements: 8.6_

- [x] 2. Implement SHA-256 checksum calculator
  - [x] 2.1 Create `src/updater/checksum.rs` with `ChecksumCalculator` struct
    - Implement `calculate(content: &[u8]) -> String` using `sha2` crate
    - Implement `calculate_all(project_dir: &Path, files: &[String]) -> Result<HashMap<String, String>>`
    - _Requirements: 6.1, 6.2, 6.3_
  - [x] 2.2 Write property test for SHA-256 checksum determinism
    - **Property 5: SHA-256 checksum determinism**
    - Generate random byte sequences, verify `calculate` returns same result on repeated calls and different results for different inputs
    - **Validates: Requirements 6.1, 6.2, 6.3**
  - [x] 2.3 Create `src/updater/mod.rs` to expose the updater module
    - _Requirements: 6.1_

- [x] 3. Implement user configuration file loader
  - [x] 3.1 Create `src/config/user_config.rs` with `UserConfig` struct and `load()` method
    - Parse `~/.axum-app-create.toml` using `toml` crate and `dirs` crate for home directory
    - Return `Default` on missing file or parse error (with warning on invalid TOML)
    - _Requirements: 7.1, 7.2, 7.3, 7.4_
  - [x] 3.2 Write property test for user config parsing
    - **Property 12: User config parsing**
    - Generate random valid TOML strings with `template_dir`, verify correct parsing; generate invalid TOML, verify default returned
    - **Validates: Requirements 7.2, 7.3, 7.4**
  - [x] 3.3 Wire `UserConfig::load()` into `src/main.rs` startup, pass `template_dir` to generation flow
    - _Requirements: 7.1, 7.5_
  - [x] 3.4 Write property test for configuration priority
    - **Property 13: Configuration priority**
    - Generate random CLI/config/default value combinations, verify CLI > config > default
    - **Validates: Requirements 1.10, 7.5**

- [x] 4. Checkpoint - Ensure all tests pass
  - Ensure all tests pass (including existing 70 tests), ask the user if questions arise.

- [x] 5. Implement custom template loader
  - [x] 5.1 Create `src/template/custom_loader.rs` with `CustomTemplateLoader` struct
    - Implement `load(dir: &Path) -> Result<HashMap<String, String>>` to recursively scan `.hbs` files
    - Validate directory exists, handle empty directory case
    - _Requirements: 1.1, 1.5, 1.6_
  - [x] 5.2 Create `src/template/resolver.rs` with `TemplateResolver` struct
    - Implement `resolve(mode, ci_enabled) -> Result<HashMap<String, ResolvedTemplate>>`
    - Merge built-in templates with custom templates: custom overrides matching paths, adds new paths
    - Define `ResolvedTemplate` struct
    - _Requirements: 1.2, 1.3, 2.5_
  - [x] 5.3 Write property test for template resolution
    - **Property 1: Template resolution override and merge**
    - Generate random built-in and custom template sets, verify merge correctness
    - **Validates: Requirements 1.2, 1.3, 2.5**

- [x] 6. Implement template inheritance system
  - [x] 6.1 Create `src/template/inheritance.rs` with `InheritanceProcessor` struct
    - Implement `parse_extends(content: &str) -> Option<String>` to extract extends directive
    - Implement `parse_overrides(content: &str) -> HashMap<String, String>` to extract override blocks
    - Implement `apply_inheritance(base_content: &str, overrides: &HashMap<String, String>) -> Result<String>`
    - _Requirements: 2.1, 2.3, 2.4, 2.7, 2.8, 2.9_
  - [x] 6.2 Write property test for extends directive parsing
    - **Property 3: Extends directive parsing**
    - Generate random path strings, construct extends directives, verify parsing
    - **Validates: Requirements 2.1**
  - [x] 6.3 Write property test for block inheritance
    - **Property 2: Block default content preservation in inheritance**
    - Generate random base templates with blocks and child templates with partial overrides, verify merge
    - **Validates: Requirements 2.2, 2.3, 2.4, 2.9**
  - [x] 6.4 Integrate `InheritanceProcessor` into `TemplateResolver::resolve()`
    - After merging custom templates, process extends directives for templates that have them
    - _Requirements: 2.1, 2.5_

- [x] 7. Refactor built-in templates with block definitions
  - [x] 7.1 Add `{{#block "name"}}...{{/block}}` markers to `single_mode/src/main.rs.hbs`
    - Define blocks: `imports`, `config`, `routes`, `middleware`, `main_body`
    - _Requirements: 3.1_
  - [x] 7.2 Add block markers to `workspace_mode/api/src/main.rs.hbs`
    - Same block set as single mode
    - _Requirements: 3.2_
  - [x] 7.3 Add `dependencies` and `dev_dependencies` blocks to `Cargo.toml.hbs` templates (single and workspace)
    - _Requirements: 3.3_
  - [x] 7.4 Register `block` and `override` as custom Handlebars helpers in `src/template/engine.rs`
    - `block` helper: renders default content, can be replaced by override
    - `override` helper: stores content for later replacement
    - _Requirements: 2.2, 2.6, 3.5_
  - [x] 7.5 Write property test for block refactoring backward compatibility
    - **Property 4: Block refactoring backward compatibility**
    - For multiple ProjectConfig combinations, verify block-refactored templates produce identical output
    - **Validates: Requirements 3.4, 9.1, 9.2, 9.4**

- [x] 8. Checkpoint - Ensure all tests pass
  - Ensure all tests pass (especially backward compatibility), ask the user if questions arise.

- [x] 9. Implement generation metadata
  - [x] 9.1 Create `src/updater/metadata.rs` with `GenerationMetadata` struct and `MetadataManager`
    - Implement `create()`, `read()`, `update()` methods
    - Use serde_json pretty print for serialization
    - _Requirements: 5.1, 5.2, 5.3_
  - [x] 9.2 Write property test for metadata serialization round-trip
    - **Property 9: Metadata serialization round-trip**
    - Generate random GenerationMetadata, serialize then deserialize, verify equivalence
    - **Validates: Requirements 4.2, 5.1, 5.2**
  - [x] 9.3 Integrate metadata creation into `generate_project()` in `src/generator/project.rs`
    - After generating all files, compute checksums and write `.axum-app-create.json`
    - Add `.axum-app-create.json` to generated `.gitignore`
    - _Requirements: 5.1, 5.4_
  - [x] 9.4 Write property test for metadata checksum completeness
    - **Property 10: Metadata contains checksums for all generated files**
    - Generate projects with various configs, verify metadata has checksum for every file
    - **Validates: Requirements 5.2, 5.4**

- [x] 10. Implement template exporter (init-template subcommand)
  - [x] 10.1 Create `src/template/exporter.rs` with `TemplateExporter` struct
    - Implement `export(mode: ProjectMode, output_dir: &Path) -> Result<()>`
    - Write all built-in templates for the given mode to the output directory
    - _Requirements: 1.7, 1.8_
  - [x] 10.2 Write property test for template export round-trip
    - **Property 11: Template export round-trip**
    - Export templates, reload as custom, verify identical content
    - **Validates: Requirements 1.7, 1.8**

- [x] 11. Implement update engine
  - [x] 11.1 Create `src/updater/engine.rs` with `UpdateEngine` struct
    - Implement `update(interactive: bool) -> Result<UpdateReport>`
    - Read metadata, regenerate templates, classify files (skip/auto-update/conflict)
    - Handle `--dry-run` (no file writes) and `--force` (overwrite all)
    - _Requirements: 4.1, 4.2, 4.4, 4.5, 4.6, 4.7, 4.8, 4.9_
  - [x] 11.2 Define `UpdateReport` struct with `files_updated`, `files_skipped`, `files_conflicted`, `files_created`
    - _Requirements: 4.4_
  - [x] 11.3 Write property test for update classification
    - **Property 6: Update classification correctness**
    - Generate random file states, verify classification logic
    - **Validates: Requirements 4.5, 4.6, 6.2, 6.3**
  - [x] 11.4 Write property test for dry-run
    - **Property 7: Dry-run does not modify files**
    - Create project in temp dir, run dry-run update, verify no files changed
    - **Validates: Requirements 4.8**
  - [x] 11.5 Write property test for force update
    - **Property 8: Force overwrites all differing files**
    - Create project, modify files, run force update, verify all updated
    - **Validates: Requirements 4.9**

- [x] 12. Refactor CLI to subcommand architecture
  - [x] 12.1 Refactor `src/main.rs` to use clap subcommands (`new`, `init-template`, `update`)
    - Maintain backward compatibility: no subcommand defaults to `new` behavior
    - Add `--template-dir` to `new` subcommand
    - Add `--dry-run` and `--force` to `update` subcommand
    - Add `--mode` to `init-template` subcommand
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5_
  - [x] 12.2 Wire `init-template` subcommand to `TemplateExporter::export()`
    - _Requirements: 1.7, 8.2_
  - [x] 12.3 Wire `update` subcommand to `UpdateEngine::update()`
    - _Requirements: 4.1, 8.3, 8.4_
  - [x] 12.4 Wire `--template-dir` (from CLI flag or UserConfig) into `TemplateResolver` in the `new` flow
    - _Requirements: 1.1, 1.10, 7.5_
  - [x] 12.5 Update `generate_project()` to accept `TemplateResolver` and use resolved templates instead of directly calling registry functions
    - _Requirements: 1.2, 1.3, 5.1_

- [x] 13. Checkpoint - Ensure all tests pass
  - Ensure all tests pass (all 70 existing + new tests), ask the user if questions arise.

- [x] 14. Integration tests for new features
  - [x] 14.1 Write integration test: generate project with custom template directory overriding a built-in template
    - Verify custom content appears in output
    - _Requirements: 1.1, 1.2_
  - [x] 14.2 Write integration test: generate project with template inheritance (extends + override blocks)
    - Verify override blocks replace base content, non-overridden blocks retain defaults
    - _Requirements: 2.1, 2.3, 2.4_
  - [x] 14.3 Write integration test: generate project then update with no changes (all files skipped)
    - _Requirements: 4.5_
  - [x] 14.4 Write integration test: generate project, modify a file, update with conflict detection
    - _Requirements: 4.7, 6.3_
  - [x] 14.5 Write integration test: init-template exports templates, use them as custom templates to generate identical project
    - _Requirements: 1.7, 1.8_
  - [x] 14.6 Write integration test: backward compatibility — all v0.2.0 argument combinations produce same results
    - _Requirements: 9.1, 9.2_

- [x] 15. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes / 备注

- All tasks are required (comprehensive implementation)
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties (proptest, min 100 iterations)
- Unit tests validate specific examples and edge cases
- The CLI refactoring (task 12) is placed after all core modules are implemented to minimize integration risk
