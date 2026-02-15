# Changelog

All notable changes to this project will be documented in this file.
本项目的所有重要更改都将记录在此文件中。

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
格式基于 [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)，
本项目遵循 [语义化版本](https://semver.org/spec/v2.0.0.html)。

---

## [Unreleased]

### Planned / 计划中
- Plugin system / 插件系统
- Template marketplace / 模板市场
- GitLab CI integration / GitLab CI 集成
- Deployment helpers / 部署助手

---

## [0.3.0] - 2026-02-15

### Added / 新增

#### Custom Template System / 自定义模板系统
- `--template-dir <PATH>` flag to load custom templates from external directories / 从外部目录加载自定义模板
- `CustomTemplateLoader` module for recursive `.hbs` file scanning / 递归扫描 `.hbs` 文件的自定义模板加载器
- `TemplateResolver` module for merging built-in and custom templates with priority / 合并内置和自定义模板的模板解析器
- `init-template` subcommand to export built-in templates for customization / 导出内置模板的子命令
- `TemplateExporter` module for writing built-in templates to filesystem / 将内置模板写入文件系统

#### Template Inheritance / 模板继承
- `InheritanceProcessor` module with `extends`/`block`/`override` directive support / 支持 extends/block/override 指令的继承处理器
- `{{!-- extends: <base_template_path> --}}` directive for child templates / 子模板的 extends 指令
- `{{#block "name"}}...{{/block}}` for defining overridable blocks in base templates / 基础模板中定义可覆盖块
- `{{#override "name"}}...{{/override}}` for replacing block content in child templates / 子模板中替换块内容
- Built-in templates refactored with blocks: `imports`, `config`, `routes`, `middleware`, `main_body`, `dependencies`, `dev_dependencies` / 内置模板重构添加块定义

#### Project Update Mechanism / 项目更新机制
- `update` subcommand to re-apply templates to existing projects / 更新已生成项目的子命令
- `UpdateEngine` with dry-run and force modes / 支持 dry-run 和 force 模式的更新引擎
- `GenerationMetadata` (`.axum-app-create.json`) for tracking project generation state / 跟踪项目生成状态的元数据
- `ChecksumCalculator` with SHA-256 for user modification detection / 使用 SHA-256 检测用户修改
- File classification: skip, auto-update, conflict, new / 文件分类：跳过、自动更新、冲突、新增

#### User Configuration / 用户配置
- `~/.axum-app-create.toml` user-level configuration file support / 用户级配置文件支持
- `UserConfig` module with TOML parsing and graceful fallback / TOML 解析和优雅降级
- Configuration priority: CLI flags > user config > defaults / 配置优先级：CLI 标志 > 用户配置 > 默认值

#### CLI Improvements / CLI 改进
- Subcommand architecture: `new`, `init-template`, `update` / 子命令架构
- Backward compatible: no subcommand defaults to `new` / 向后兼容：无子命令默认为 new

#### Testing / 测试
- 13 property-based tests with `proptest` covering template resolution, inheritance, checksums, update logic, metadata serialization, and configuration priority / 13 个属性测试
- 34 integration tests (up from 22) / 34 个集成测试（从 22 个增加）
- 100 unit tests (up from 56) / 100 个单元测试（从 56 个增加）
- Total: 137 tests passing / 共计：137 个测试通过

### Changed / 更改
- Version bumped to 0.3.0 / 版本升级到 0.3.0
- CLI argument parsing moved to subcommand-based architecture / CLI 参数解析改为子命令架构
- Generator now creates `.axum-app-create.json` metadata and adds it to `.gitignore` / 生成器现在创建元数据文件
- Built-in templates refactored with block definitions (backward compatible) / 内置模板重构添加块定义（向后兼容）

---

## [0.2.0] - 2026-02-11

### Added / 新增

#### Workspace Mode / 工作区模式
- Multi-crate workspace project generation (Clean Architecture) / 多 crate 工作区项目生成（分层架构）
- Four-crate structure: `api`, `domain`, `infrastructure`, `common` / 四 crate 结构
- `--mode workspace` CLI flag / `--mode workspace` 命令行标志
- Workspace-aware Dockerfile with multi-stage build / 适配工作区的多阶段构建 Dockerfile
- Workspace README with architecture overview / 包含架构概览的工作区 README

#### Configuration Presets / 配置预设
- `--preset minimal` — no optional features / 无可选功能
- `--preset api` — PostgreSQL + Auth + Biz-error / PostgreSQL + 认证 + 业务错误
- `--preset fullstack` — Both databases + all features / 双数据库 + 全部功能
- Preset resolution priority: individual CLI flags > preset > interactive prompt > defaults / 预设解析优先级：单独 CLI 标志 > 预设值 > 交互式提示 > 默认值

#### CI/CD Integration / CI/CD 集成
- `--ci` flag to generate GitHub Actions workflow / `--ci` 标志生成 GitHub Actions 工作流
- CI workflow with 4 jobs: check, test, fmt, clippy / 包含 4 个 job 的 CI 工作流
- Workspace-aware CI (`--workspace` flag auto-applied) / 工作区感知的 CI（自动应用 `--workspace` 标志）
- Rust toolchain caching with `Swatinem/rust-cache@v2` / 使用 rust-cache 缓存工具链

#### Testing / 测试
- 12 new integration tests for workspace mode, presets, and CI / 12 个新集成测试
- Property-based tests with `proptest` for preset mapping and override logic / 使用 proptest 的属性测试
- Compile verification tests for generated workspace projects / 生成工作区项目的编译验证测试

### Changed / 更改
- Version bumped to 0.2.0 / 版本升级到 0.2.0
- Interactive prompts now include mode, preset, and CI selection / 交互式提示现在包含模式、预设和 CI 选择
- Template context extended with `is_workspace` and `has_ci` fields / 模板上下文扩展了 `is_workspace` 和 `has_ci` 字段
- Generator flow updated to select templates by project mode / 生成器流程更新为按项目模式选择模板

---

## [0.1.0] - 2025-02-06

### Added / 新增

#### Core Features / 核心功能
- CLI tool for scaffolding Axum web applications / 用于搭建 Axum Web 应用的 CLI 工具
- Interactive prompt system with inquire / 使用 inquire 的交互式提示系统
- Project name validation with Cargo naming conventions / 符合 Cargo 命名规范的项目名称验证
- Automatic git repository initialization / 自动初始化 Git 仓库
- Bilingual error messages and documentation (English + Chinese) / 双语错误消息和文档（英文+中文）
- Non-interactive mode for CI/CD automation / 用于 CI/CD 自动化的非交互模式

#### Template Engine / 模板引擎
- Handlebars-based template rendering with strict mode / 基于 Handlebars 的严格模式模板渲染
- Template context builder with project name variants / 支持项目名称变体的模板上下文构建器
- Helper functions for common transformations / 常用转换的辅助函数

#### Optional Features / 可选功能
- **Database Support / 数据库支持**:
  - PostgreSQL with SQLx / 使用 SQLx 的 PostgreSQL 支持
  - SQLite for development / 用于开发的 SQLite 支持
  - Database migration templates / 数据库迁移模板
  - Connection pooling configuration / 连接池配置
- **JWT Authentication / JWT 认证**:
  - User registration endpoint (/register) / 用户注册端点
  - User login endpoint (/login) / 用户登录端点
  - JWT token generation and validation / JWT 令牌生成和验证
  - Protected route example / 受保护路由示例
- **Business Error Handling / 业务错误处理**:
  - YAML-based error definitions / 基于 YAML 的错误定义
  - i18n support (en/zh) for error messages / 错误消息的国际化支持
  - Predefined error codes (USER_NOT_FOUND, AUTH_FAILED, etc.) / 预定义错误码
- **Structured Logging / 结构化日志**:
  - Configurable log levels / 可配置的日志级别
  - Tracing integration / Tracing 集成
  - Environment-based configuration / 基于环境的配置

#### Generated Project Structure / 生成的项目结构
- Cargo.toml with feature flags / 带有功能标志的 Cargo.toml
- Main application entry point with router setup / 带有路由设置的主应用入口
- Configuration module with environment variables / 带有环境变量的配置模块
- Handler modules for API endpoints / API 端点的处理模块
- Health check endpoint / 健康检查端点
- .env.example with all configuration options / 包含所有配置选项的 .env.example
- .gitignore optimized for Rust projects / 为 Rust 项目优化的 .gitignore
- Comprehensive README.md with quick start guide / 包含快速入门指南的全面 README.md
- Integration test suite / 集成测试套件

#### Documentation / 文档
- Comprehensive CLI tool README.md / 全面的 CLI 工具 README.md
- Bilingual quick start guide / 双语快速入门指南
- Command-line options reference / 命令行选项参考
- Usage examples for all features / 所有功能的使用示例
- Troubleshooting section / 故障排查部分
- Contributing guidelines / 贡献指南
- This CHANGELOG.md / 此 CHANGELOG.md 文件

#### Testing / 测试
- Unit tests for all core modules / 所有核心模块的单元测试
- Integration tests for CLI functionality / CLI 功能的集成测试
- Template rendering tests / 模板渲染测试
- Project generation tests / 项目生成测试
- Test fixtures for validation / 验证用的测试夹具
- Total: 36 tests passing / 共计：36 个测试通过

#### Developer Experience / 开发者体验
- ASCII art success messages / ASCII 艺术成功消息
- Generation timestamps / 生成时间戳
- Detailed next steps guidance / 详细的后续步骤指导
- Error messages with troubleshooting suggestions / 带有故障排查建议的错误消息
- Progress indicators during generation / 生成过程中的进度指示器

### Changed / 更改
- N/A (initial release / 初始版本)

### Deprecated / 废弃
- N/A (initial release / 初始版本)

### Removed / 移除
- N/A (initial release / 初始版本)

### Fixed / 修复
- Fixed template rendering error for `project_snake_case` variable / 修复 `project_snake_case` 变量的模板渲染错误
- Fixed conditional variable names in templates (`has_database_postgres` → `has_postgresql`) / 修复模板中的条件变量名称

### Security / 安全
- JWT_SECRET clearly marked as example in templates / 模板中 JWT_SECRET 明确标记为示例
- .env files added to generated .gitignore / .env 文件添加到生成的 .gitignore
- No hardcoded credentials in templates or code / 模板或代码中无硬编码凭据

---

## [0.0.1] - 2025-02-05

### Added / 新增
- Initial project structure / 初始项目结构
- Basic CLI argument parsing with clap / 使用 clap 的基本命令行参数解析
- Foundation for template system / 模板系统基础

---

## Release Notes / 发布说明

### Version 0.1.0 (Phase 1 MVP) / 版本 0.1.0 (Phase 1 MVP)

**Status / 状态**: ✅ Complete / 完成

This release represents the Phase 1 MVP of axum-app-create. It provides a solid foundation for scaffolding Axum web applications with sensible defaults and optional features.
此版本代表 axum-app-create 的 Phase 1 MVP。它为使用合理默认值和可选功能搭建 Axum Web 应用提供了坚实的基础。

#### Key Capabilities / 主要能力
- ✅ Generate single-package Axum applications / 生成单包 Axum 应用
- ✅ Interactive and non-interactive modes / 交互式和非交互式模式
- ✅ Optional features: database, auth, logging, biz-error / 可选功能：数据库、认证、日志、业务错误
- ✅ Bilingual support (English + Chinese) / 双语支持（英文+中文）
- ✅ Git initialization / Git 初始化
- ✅ Comprehensive documentation / 全面的文档

#### Known Limitations / 已知限制
- Only single-package projects (workspace mode planned) / 仅支持单包项目（工作区模式已计划）
- No custom templates (planned for v0.2.0) / 无自定义模板（计划于 v0.2.0）
- No project update mechanism (planned for v0.2.0) / 无项目更新机制（计划于 v0.2.0）

#### Migration Guide / 迁移指南
N/A (initial release / 初始版本)

---

## Links / 链接

- **GitHub Repository**: https://github.com/Yu-Xiao-Sheng/axum-app-create
- **Issue Tracker**: https://github.com/Yu-Xiao-Sheng/axum-app-create/issues
- **Documentation**: https://github.com/Yu-Xiao-Sheng/axum-app-create/blob/main/README.md

---

**Note / 注意**:
- Versions following [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html) / 遵循[语义化版本 2.0.0](https://semver.org/spec/v2.0.0.html)的版本
- All dates in UTC / 所有日期为 UTC 时区
- Questions? Please open an issue / 有问题？请创建 issue
