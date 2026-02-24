# axum-app-create

> 快速创建 Axum Web 应用的命令行工具 / A command-line tool to quickly create Axum web applications with zero configuration

[![CI](https://github.com/Yu-Xiao-Sheng/axum-app-create/workflows/CI/badge.svg)](https://github.com/Yu-Xiao-Sheng/axum-app-create/actions)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)

`axum-app-create` 是一个用于快速搭建 Axum Web 应用的 CLI 工具，类似于 `create-react-app` 或 `cargo-generate`。它能生成生产就绪的项目，包含合理的默认配置、结构化日志、错误处理以及可选的数据库支持和身份认证功能。

`axum-app-create` is a CLI tool that helps you quickly scaffold Axum web applications in Rust. It generates production-ready projects with sensible defaults, structured logging, error handling, and optional features like database support and JWT authentication.

---

## 📚 Table of Contents / 目录

- [Features / 功能特性](#-features-功能特性)
- [Installation / 安装](#-installation-安装)
- [Quick Start / 快速开始](#-quick-start-快速开始)
- [Command-Line Options / 命令行选项](#-command-line-options-命令行选项)
- [Examples / 使用示例](#-examples-使用示例)
- [Development / 开发指南](#-development-开发指南)
- [Troubleshooting / 故障排查](#-troubleshooting-故障排查)
- [Roadmap / 路线图](#-roadmap-路线图)
- [Contributing / 贡献指南](#-contributing-贡献指南)
- [License / 许可证](#-license-许可证)

---

## ✨ Features / 功能特性

### Core Features / 核心功能

- ✅ **Single Project Mode / 单项目模式**: Generate single-package Axum applications / 生成单包 Axum 应用
- ✅ **Workspace Mode / 工作区模式**: Generate multi-crate workspace projects (Clean Architecture) / 生成多 crate 工作区项目（分层架构）
- ✅ **Configuration Presets / 配置预设**: Quick setup with `--preset minimal/api/fullstack` / 使用预设快速配置
- ✅ **CI/CD Integration / CI/CD 集成**: Generate GitHub Actions workflow with `--ci` / 生成 GitHub Actions 工作流
- ✅ **Custom Templates / 自定义模板**: Override or extend built-in templates with `--template-dir` / 使用自定义模板覆盖或扩展内置模板
- ✅ **Template Inheritance / 模板继承**: Extend base templates with `extends`/`block`/`override` directives / 通过继承指令扩展基础模板
- ✅ **Project Update / 项目更新**: Update existing projects with `axum-app-create update` / 使用 update 子命令更新已有项目
- ✅ **User Configuration / 用户配置**: Default settings via `~/.axum-app-create.toml` / 通过用户配置文件设置默认值
- ✅ **Interactive Prompts / 交互式提示**: Friendly CLI with interactive configuration / 友好的交互式配置界面
- ✅ **Sensible Defaults / 合理默认值**: Works out of the box with zero configuration / 开箱即用，零配置
- ✅ **Production-Ready Templates / 生产就绪模板**: Includes tracing, error handling, proper structure / 包含日志、错误处理、规范结构
- ✅ **Bilingual Documentation / 双语文档**: English and Chinese (中文) support / 英文和中文支持
- ✅ **Git Initialization / Git 初始化**: Automatic git repo, .gitignore, and initial commit / 自动初始化 Git 仓库
- ✅ **Plugin System / 插件系统**: Extend tool capabilities with installable plugins / 通过可安装插件扩展工具能力

### Optional Features / 可选功能

- 🗄️ **Database Support / 数据库支持**: PostgreSQL, SQLite, or both / 支持 PostgreSQL、SQLite 或两者
- 🔐 **JWT Authentication / JWT 认证**: Built-in user registration and login endpoints / 内置用户注册和登录端点
- 📝 **Business Error Handling / 业务错误处理**: YAML-based i18n error definitions / 基于 YAML 的国际化错误定义
- 📊 **Structured Logging / 结构化日志**: Configurable log levels with tracing / 可配置的日志级别

---

## 📦 Installation / 安装

### Prerequisites / 前置要求

- Rust toolchain 1.85+ / Rust 工具链 1.85+
- Git (for project initialization / 用于项目初始化)

Install Rust / 安装 Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### From crates.io (Recommended / 推荐)

```bash
cargo install axum-app-create
```

### From Source / 从源码安装

```bash
git clone https://github.com/Yu-Xiao-Sheng/axum-app-create.git
cd axum-app-create
cargo install --path .
```

### Build from Source / 从源码构建

```bash
git clone https://github.com/Yu-Xiao-Sheng/axum-app-create.git
cd axum-app-create
cargo build --release

# Binary location / 二进制文件位置:
# Linux/macOS: ./target/release/axum-app-create
# Windows: .\target\release\axum-app-create.exe
```

---

## 🚀 Quick Start / 快速开始

### Interactive Mode / 交互式模式 (Recommended / 推荐)

```bash
axum-app-create my-app
# Follow the prompts / 按照提示配置项目
cd my-app
cargo run
```

The CLI will prompt you to configure / CLI 将提示您配置:
- Database support (PostgreSQL, SQLite, or none) / 数据库支持
- Authentication (JWT endpoints) / 认证功能
- Logging level / 日志级别
- Business error handling / 业务错误处理

### With All Features / 包含所有功能

```bash
axum-app-create my-app \
  --database both \
  --auth \
  --biz-error \
  --log-level info

cd my-app
cargo run
```

### Workspace Mode / 工作区模式

```bash
axum-app-create my-app --mode workspace --preset fullstack --ci
cd my-app
cargo run
```

### Using Presets / 使用预设

```bash
# Minimal - no optional features / 最小配置
axum-app-create my-app --preset minimal

# API - PostgreSQL + Auth + Biz-error / API 开发
axum-app-create my-app --preset api

# Fullstack - Both DBs + all features / 全栈开发
axum-app-create my-app --preset fullstack
```

### Minimal Project / 最小项目

```bash
axum-app-create minimal-app --non-interactive
cd minimal-app
cargo run
```

### Test Your API / 测试 API

```bash
# Health check / 健康检查
curl http://127.0.0.1:8080/health

# Expected response / 预期响应:
{"status":"ok","version":"0.1.0"}
```

---

## 🎛️ Command-Line Options / 命令行选项

### Full Usage / 完整用法

```bash
axum-app-create [COMMAND] [OPTIONS] [PROJECT_NAME]
```

### Subcommands / 子命令

| Command / 命令 | Description / 描述 |
|----------------|-------------------|
| `new` | Create a new project (default if no subcommand) / 创建新项目（默认） |
| `init-template` | Export built-in templates for customization / 导出内置模板 |
| `update` | Update a previously generated project / 更新已生成的项目 |
| `plugin install` | Install a plugin from local path or Git / 从本地路径或 Git 安装插件 |
| `plugin uninstall` | Uninstall a plugin / 卸载插件 |
| `plugin enable` | Enable a disabled plugin / 启用已禁用的插件 |
| `plugin disable` | Disable a plugin / 禁用插件 |
| `plugin list` | List installed plugins / 列出已安装插件 |
| `plugin info` | Show plugin details / 显示插件详情 |
| `plugin run` | Run a plugin command / 运行插件命令 |

### Options / 选项

| Flag / 标志 | Description / 描述 | Default / 默认值 |
|-------------|-------------------|------------------|
| `[PROJECT_NAME]` | Project name (positional argument) / 项目名称（位置参数） | Prompted / 提示输入 |
| `--mode <MODE>` | Project mode: `single`, `workspace` / 项目模式 | `single` |
| `--preset <PRESET>` | Preset: `minimal`, `api`, `fullstack` / 配置预设 | Prompted / 提示输入 |
| `--database <TYPE>` | Database: `none`, `postgresql`, `sqlite`, `both` / 数据库类型 | Prompted / 提示输入 |
| `--auth` | Enable JWT authentication / 启用 JWT 认证 | Prompted / 提示输入 |
| `--biz-error` | Enable business error handling / 启用业务错误处理 | Prompted / 提示输入 |
| `--log-level <LEVEL>` | Logging: `trace`, `debug`, `info`, `warn`, `error` / 日志级别 | Prompted / 提示输入 |
| `--ci` | Generate GitHub Actions CI workflow / 生成 CI 工作流 | `false` |
| `--author <NAME>` | Author name for generated project / 项目作者名称 | Git config / Git 配置 |
| `--force` | Force overwrite if target directory exists / 强制覆盖已存在的目录 | `false` |
| `--non-interactive` | Disable prompts / 禁用交互提示 (fail if required values missing / 缺少必需值时失败) | `false` |
| `--template-dir <DIR>` | Custom template directory / 自定义模板目录 | None |
| `--help`, `-h` | Show help message / 显示帮助信息 | - |
| `--version`, `-V` | Show version / 显示版本 | - |

### Examples / 示例

```bash
# Interactive with all features / 交互式启用所有功能
axum-app-create myapp

# PostgreSQL only / 仅 PostgreSQL
axum-app-create myapp --database postgresql

# With auth and logging / 包含认证和日志
axum-app-create myapp --auth --log-level debug

# Minimal, non-interactive / 最小化、非交互式
axum-app-create myapp --non-interactive

# Force overwrite existing project / 强制覆盖已存在的项目
axum-app-create myapp --force

# Using presets / 使用预设
axum-app-create myapp --preset api
axum-app-create myapp --preset fullstack --ci

# Workspace mode / 工作区模式
axum-app-create myapp --mode workspace
axum-app-create myapp --mode workspace --preset api --ci

# Full featured / 完整功能
axum-app-create myapp \
  --mode workspace \
  --preset fullstack \
  --ci \
  --author "Your Name"

# Custom templates (v0.3.0) / 自定义模板
axum-app-create init-template ./my-templates --mode single
axum-app-create new myapp --template-dir ./my-templates

# Update existing project (v0.3.0) / 更新已有项目
axum-app-create update ./myapp --dry-run
axum-app-create update ./myapp --force

# Plugin management (v0.4.0) / 插件管理
axum-app-create plugin install ./my-plugin
axum-app-create plugin install https://github.com/user/plugin.git --git
axum-app-create plugin list
axum-app-create plugin enable my-plugin
axum-app-create plugin disable my-plugin
axum-app-create plugin info my-plugin
axum-app-create plugin uninstall my-plugin
axum-app-create plugin run my-plugin gen-schema
```

---

## 📖 Examples / 使用示例

### Example 1: Simple API Server / 简单 API 服务器

```bash
axum-app-create simple-api --non-interactive
cd simple-api
cargo run
```

Generates / 生成:
- Basic Axum server / 基础 Axum 服务器
- Health check endpoint / 健康检查端点
- Structured logging / 结构化日志
- Error handling / 错误处理
- Git repository / Git 仓库

### Example 2: Full-Stack Application / 全栈应用

```bash
axum-app-create fullstack-app \
  --database postgresql \
  --auth \
  --biz-error \
  --log-level info

cd fullstack-app

# Configure environment / 配置环境
cp .env.example .env
# Edit .env with your database URL and JWT secret / 编辑数据库 URL 和 JWT 密钥

# Run migrations (if database enabled) / 运行迁移
cargo run

# Test the API / 测试 API
curl http://127.0.0.1:8080/health
curl -X POST http://127.0.0.1:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"password123"}'
```

### Example 3: SQLite for Development / SQLite 开发环境

```bash
axum-app-create dev-app --database sqlite --log-level debug
cd dev-app
cargo run
```

Perfect for / 适用于:
- Local development / 本地开发
- Small applications / 小型应用
- Quick prototypes / 快速原型

### Example 4: Custom Author Name / 自定义作者名称

```bash
axum-app-create myapp --author "Jane Developer" --non-interactive
```

---

## 🛠️ Development / 开发指南

### Prerequisites / 前置要求

- Rust 1.85+ / Rust 工具链 1.85+
- Git / 版本控制

### Build Commands / 构建命令

```bash
# Clone repository / 克隆仓库
git clone https://github.com/Yu-Xiao-Sheng/axum-app-create.git
cd axum-app-create

# Development build / 开发构建
cargo build

# Release build / 发布构建
cargo build --release

# Run CLI / 运行 CLI
cargo run -- --help
```

### Testing / 测试

```bash
# Run all tests / 运行所有测试
cargo test

# Run tests with output / 运行测试并显示输出
cargo test -- --nocapture

# Run specific test / 运行特定测试
cargo test test_project_name_validation
```

### Code Quality / 代码质量

```bash
# Format code / 格式化代码
cargo fmt

# Check formatting / 检查格式
cargo fmt -- --check

# Run linter / 运行代码检查
cargo clippy -- -D warnings

# Fix lints automatically / 自动修复
cargo clippy --fix
```

### Documentation / 文档

```bash
# Generate and open documentation / 生成并打开文档
cargo doc --open
```

---

## 🔧 Troubleshooting / 故障排查

### Common Issues / 常见问题

#### Issue: "Permission denied" when creating project / 创建项目时权限被拒绝

**Error / 错误**:
```
❌ 权限拒绝 / Permission denied: 无法访问目录 / cannot access directory
```

**Solutions / 解决方案**:
1. Check directory permissions / 检查目录权限: `ls -la /path/to/parent`
2. Use a directory you have write access to / 使用有写权限的目录: `cd ~`
3. Use temporary directory / 使用临时目录: `/tmp/my-project`
4. Check available disk space / 检查可用磁盘空间: `df -h`

#### Issue: "Directory already exists" / 目录已存在

**Error / 错误**:
```
❌ 目录已存在 / Directory already exists: 'my-app'
```

**Solutions / 解决方案**:
1. Use `--force` flag to overwrite / 使用 `--force` 标志覆盖: `axum-app-create my-app --force`
2. Remove existing directory / 删除现有目录: `rm -rf my-app`
3. Use different name / 使用不同名称: `axum-app-create my-app-2`
4. Interactive mode will prompt to overwrite / 交互式模式会提示是否覆盖

#### Issue: Generated project fails to compile / 生成的项目编译失败

**Error / 错误**:
```
error: linking with `cc` failed
```

**Solutions / 解决方案**:
1. Update Rust toolchain / 更新 Rust 工具链: `rustup update`
2. Clean and rebuild / 清理并重新构建: `cd my-app && cargo clean && cargo build`
3. Check system dependencies / 检查系统依赖:
   - Linux: `sudo apt install build-essential pkg-config`
   - macOS: `xcode-select --install`
   - Windows: Install [Build Tools](https://visualstudio.microsoft.com/downloads/)

#### Issue: Database connection fails / 数据库连接失败

**Error / 错误**:
```
Error: Database connection failed
```

**Solutions / 解决方案**:
1. Verify DATABASE_URL in .env / 验证 .env 中的 DATABASE_URL
2. Check database is running / 检查数据库是否运行:
   - PostgreSQL: `sudo service postgresql status`
   - SQLite: No service needed / 无需服务
3. Test connection manually / 手动测试连接:
   - PostgreSQL: `psql $DATABASE_URL`
   - Run migrations / 运行迁移: `cargo run -- migrate`

#### Issue: Template rendering errors / 模板渲染错误

**Error / 错误**:
```
Error: Failed to render template
```

**Solutions / 解决方案**:
1. Check project name follows Cargo naming conventions / 检查项目名称是否符合 Cargo 命名规范
2. Report bug with: `RUST_LOG=debug axum-app-create myapp > debug.log`
3. Open issue on GitHub / 在 GitHub 上提交问题

### Debug Mode / 调试模式

Enable debug logging for more details / 启用调试日志获取更多详情:

```bash
RUST_LOG=debug axum-app-create myapp
```

### Getting Help / 获取帮助

```bash
# Show help / 显示帮助
axum-app-create --help

# Show version / 显示版本
axum-app-create --version

# Report bugs / 报告 Bug
https://github.com/Yu-Xiao-Sheng/axum-app-create/issues
```

---

## 🗺️ Roadmap / 路线图

### Phase 1 MVP ✅ (Complete / 已完成)

- [x] CLI argument parsing with clap / 使用 clap 解析命令行参数
- [x] Interactive prompts with inquire / 使用 inquire 实现交互式提示
- [x] Template rendering with Handlebars / 使用 Handlebars 渲染模板
- [x] Project name validation / 项目名称验证
- [x] Embedded project templates / 嵌入式项目模板
- [x] Git initialization / Git 初始化
- [x] Optional feature templates (database, auth, logging, biz-error) / 可选功能模板
- [x] Integration tests / 集成测试
- [x] Bilingual documentation (English + Chinese) / 双语文档（英文+中文）

### Phase 2: Enhanced Features ✅ (Complete / 已完成)

- [x] Workspace mode (multi-package projects) / 工作区模式（多包项目）
- [x] Interactive configuration presets / 交互式配置预设
- [x] CI/CD integration (GitHub Actions) / CI/CD 集成（GitHub Actions）
- [x] Custom template system / 自定义模板系统
- [x] Template inheritance / 模板继承
- [x] Project update mechanism / 项目更新机制

### Phase 3: Ecosystem Integration 🔮 (Future / 未来)

- [x] Plugin system / 插件系统
- [ ] Template marketplace / 模板市场
- [ ] GitLab CI integration / GitLab CI 集成
- [ ] Deployment helpers / 部署助手

See [issues](https://github.com/Yu-Xiao-Sheng/axum-app-create/issues) for detailed progress and to suggest features.

查看 [issues](https://github.com/Yu-Xiao-Sheng/axum-app-create/issues) 了解详细进度或建议功能。

---

## 🤝 Contributing / 贡献指南

Contributions are welcome! Please feel free to submit pull requests or open issues.

欢迎贡献！请随时提交 Pull Request 或创建 Issue。

### Development Setup / 开发环境设置

```bash
# Fork and clone / Fork 并克隆仓库
git clone https://github.com/YOUR-USERNAME/axum-app-create.git
cd axum-app-create

# Install dependencies / 安装依赖
cargo build

# Run tests / 运行测试
cargo test

# Format code / 格式化代码
cargo fmt

# Run linter / 运行代码检查
cargo clippy -- -D warnings
```

### Pull Request Process / PR 流程

1. Fork the repository / Fork 仓库
2. Create a feature branch / 创建功能分支: `git checkout -b feature/amazing-feature`
3. Make your changes / 进行更改
4. Write tests for new features / 为新功能编写测试
5. Ensure all tests pass / 确保所有测试通过: `cargo test`
6. Format code / 格式化代码: `cargo fmt`
7. Run clippy / 运行 clippy: `cargo clippy -- -D warnings`
8. Commit your changes / 提交更改: `git commit -m 'Add amazing feature'`
9. Push to the branch / 推送到分支: `git push origin feature/amazing-feature`
10. Open a Pull Request / 打开 Pull Request

### Code Style / 代码风格

- Follow Rust 2024 Edition conventions / 遵循 Rust 2024 Edition 规范
- Use `cargo fmt` for formatting / 使用 `cargo fmt` 格式化
- Pass `cargo clippy -- -D warnings` / 通过 `cargo clippy -- -D warnings` 检查
- Add tests for new features / 为新功能添加测试
- Update documentation as needed / 根据需要更新文档

---

## 📄 License / 许可证

This project is licensed under either of:
本项目采用以下任一许可证:

- MIT License ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.
由您选择。

---

## 🙏 Acknowledgments / 致谢

Built with great open-source tools:
基于优秀的开源工具构建:

- [Axum](https://github.com/tokio-rs/axum) - Ergonomic and modular web framework / 优雅的模块化 Web 框架
- [Tokio](https://tokio.rs/) - Async runtime / 异步运行时
- [clap](https://github.com/clap-rs/clap) - Command-line argument parser / 命令行参数解析器
- [inquire](https://github.com/mikaelmello/inquire) - Interactive prompts / 交互式提示
- [handlebars-rust](https://github.com/sunng87/handlebars-rust) - Template engine / 模板引擎
- [SQLx](https://github.com/launchbadge/sqlx) - Database toolkit / 数据库工具包
- Inspired by / 灵感来源:
  - [create-react-app](https://github.com/facebook/create-react-app)
  - [cargo-generate](https://github.com/cargo-generate/cargo-generate)

---

**Current Version / 当前版本**: 0.4.0

**Status / 状态**: v0.4.0 Plugin System Complete / v0.4.0 插件系统已完成 ✅

**Year / 年份**: 2026

---

<p align="center">
  <i>Built with ❤️ by the Rust community / 由 Rust 社区用 ❤️ 构建</i>
</p>
