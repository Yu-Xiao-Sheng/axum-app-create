# 需求文档 / Requirements Document

## 简介 / Introduction

本文档定义了 `axum-app-create` CLI 工具 v0.4.0 版本的需求。v0.4.0 在已完成的 v0.3.0 基础上引入插件系统（Plugin System），作为工具未来可扩展性的核心架构特性。插件系统允许第三方开发者和团队通过插件扩展工具的能力，包括自定义模板集、代码生成器、后生成钩子和自定义 CLI 命令。该系统为未来的模板市场（Template Marketplace）奠定基础。

This document defines the requirements for `axum-app-create` CLI tool v0.4.0. Building on the completed v0.3.0, v0.4.0 introduces a Plugin System as the core architectural feature for future extensibility. The plugin system allows third-party developers and teams to extend the tool's capabilities through plugins, including custom template sets, code generators, post-generation hooks, and custom CLI commands. This system lays the foundation for a future Template Marketplace.

## 术语表 / Glossary

- **CLI_Tool**: `axum-app-create` 命令行工具，用于脚手架生成 Axum Web 应用项目
- **Plugin**: 一个可安装的扩展包，遵循插件接口规范，为 CLI_Tool 提供额外功能
- **PluginManager**: 插件管理器模块，负责插件的发现、加载、生命周期管理和依赖解析
- **PluginManifest**: 插件清单文件（`plugin.toml`），描述插件的元数据、能力声明和依赖关系
- **PluginRegistry**: 插件注册表，维护已安装插件的索引和状态信息
- **PluginAPI**: 插件接口层，定义插件可以扩展的能力（Hook Points）和数据交换协议
- **HookPoint**: 钩子点，CLI_Tool 生成流程中插件可以介入的扩展点（如 pre_generate、post_generate）
- **PluginSource**: 插件来源，支持本地路径、Git 仓库和 crates.io 三种分发方式
- **PluginConfig**: 插件配置，存储在用户配置文件或项目配置中的插件特定设置
- **ProjectConfig**: 项目配置结构体（`config::ProjectConfig`），包含项目名称、功能集和子配置
- **TemplateResolver**: 模板解析器，负责合并内置模板与自定义模板，处理覆盖和继承逻辑
- **Generator**: 项目生成器模块（`generator::project`），负责编排整个项目生成流程
- **UserConfig**: 用户级配置文件（`~/.axum-app-create.toml`），存储默认设置和插件配置

## 需求 / Requirements

### 需求 1：插件清单与元数据 / Plugin Manifest and Metadata

**用户故事 / User Story:** 作为一名插件开发者，我希望通过标准化的清单文件描述插件的元数据和能力，以便 CLI_Tool 能够正确识别和加载插件。

As a plugin developer, I want to describe plugin metadata and capabilities through a standardized manifest file, so that the CLI_Tool can correctly identify and load plugins.

#### 验收标准 / Acceptance Criteria

1. THE PluginManifest SHALL 使用 TOML 格式定义，文件名为 `plugin.toml`，位于插件根目录
2. THE PluginManifest SHALL 包含以下必填字段：`name`（插件名称，kebab-case）、`version`（语义化版本）、`description`（插件描述）、`min_tool_version`（最低兼容的 CLI_Tool 版本）
3. THE PluginManifest SHALL 包含以下可选字段：`author`（作者）、`license`（许可证）、`repository`（仓库地址）、`homepage`（主页）、`keywords`（关键词列表）
4. THE PluginManifest SHALL 包含 `[capabilities]` 段，声明插件提供的能力类型：`templates`（模板集）、`hooks`（生命周期钩子）、`commands`（自定义命令）
5. WHEN PluginManifest 缺少必填字段, THE PluginManager SHALL 输出包含缺失字段名称的错误信息并拒绝加载该插件
6. THE PluginManifest SHALL 支持 `[dependencies]` 段，声明对其他插件的依赖关系（插件名称和版本约束）
7. THE PluginManager SHALL 解析 PluginManifest 并将其反序列化为 `PluginManifest` 结构体
8. THE Pretty_Printer SHALL 将 `PluginManifest` 结构体格式化输出为有效的 TOML 格式
9. FOR ALL 有效的 PluginManifest 结构体，序列化为 TOML 再反序列化 SHALL 产生等价的结构体（往返属性）

### 需求 2：插件发现与加载 / Plugin Discovery and Loading

**用户故事 / User Story:** 作为一名开发者，我希望 CLI_Tool 能够从多种来源自动发现和加载插件，以便灵活地使用本地开发的或社区共享的插件。

As a developer, I want the CLI_Tool to automatically discover and load plugins from multiple sources, so that I can flexibly use locally developed or community-shared plugins.

#### 验收标准 / Acceptance Criteria

1. THE PluginManager SHALL 支持从本地文件系统路径加载插件（绝对路径或相对路径）
2. THE PluginManager SHALL 支持从 Git 仓库 URL 克隆并加载插件
3. THE PluginManager SHALL 支持从 crates.io 下载并加载 Rust crate 形式的插件
4. WHEN 加载本地路径插件, THE PluginManager SHALL 验证路径存在且包含有效的 `plugin.toml`
5. WHEN 加载 Git 仓库插件, THE PluginManager SHALL 克隆仓库到本地缓存目录（`~/.axum-app-create/plugins/`）
6. WHEN 插件来源不可达（路径不存在、Git 克隆失败、crate 下载失败）, THE PluginManager SHALL 输出包含来源信息的错误信息
7. THE PluginManager SHALL 在加载插件时验证 `min_tool_version` 与当前 CLI_Tool 版本的兼容性
8. WHEN 插件的 `min_tool_version` 高于当前 CLI_Tool 版本, THE PluginManager SHALL 输出版本不兼容的错误信息并拒绝加载
9. THE PluginManager SHALL 维护插件缓存目录（`~/.axum-app-create/plugins/`），避免重复下载

### 需求 3：插件生命周期管理 / Plugin Lifecycle Management

**用户故事 / User Story:** 作为一名开发者，我希望通过 CLI 命令管理插件的安装、启用、禁用和卸载，以便方便地控制哪些插件处于活跃状态。

As a developer, I want to manage plugin installation, enabling, disabling, and uninstallation through CLI commands, so that I can conveniently control which plugins are active.

#### 验收标准 / Acceptance Criteria

1. THE CLI_Tool SHALL 支持 `axum-app-create plugin install <SOURCE>` 命令，从指定来源安装插件
2. THE CLI_Tool SHALL 支持 `axum-app-create plugin uninstall <PLUGIN_NAME>` 命令，卸载已安装的插件
3. THE CLI_Tool SHALL 支持 `axum-app-create plugin enable <PLUGIN_NAME>` 命令，启用已安装但被禁用的插件
4. THE CLI_Tool SHALL 支持 `axum-app-create plugin disable <PLUGIN_NAME>` 命令，禁用已安装的插件（保留文件但不加载）
5. THE CLI_Tool SHALL 支持 `axum-app-create plugin list` 命令，显示所有已安装插件的名称、版本、状态（启用/禁用）和来源
6. THE CLI_Tool SHALL 支持 `axum-app-create plugin info <PLUGIN_NAME>` 命令，显示指定插件的详细信息（清单内容、能力、依赖）
7. WHEN 安装插件成功, THE PluginRegistry SHALL 在注册表文件（`~/.axum-app-create/plugins.toml`）中记录插件信息和状态
8. WHEN 卸载插件, THE PluginManager SHALL 删除插件文件并从注册表中移除记录
9. WHEN 卸载的插件被其他已安装插件依赖, THE PluginManager SHALL 输出依赖警告并要求用户确认
10. THE PluginRegistry SHALL 将注册表数据序列化为 TOML 格式存储，并能正确反序列化恢复


### 需求 4：插件 API 与钩子系统 / Plugin API and Hook System

**用户故事 / User Story:** 作为一名插件开发者，我希望有清晰定义的 API 和钩子点，以便插件能够在项目生成流程的关键阶段介入并扩展功能。

As a plugin developer, I want clearly defined APIs and hook points, so that plugins can intervene at key stages of the project generation workflow and extend functionality.

#### 验收标准 / Acceptance Criteria

1. THE PluginAPI SHALL 定义以下钩子点：`pre_generate`（生成前）、`post_generate`（生成后）、`modify_context`（修改模板上下文）、`modify_templates`（修改模板集合）
2. WHEN `pre_generate` 钩子被触发, THE PluginManager SHALL 按插件优先级顺序调用所有注册了该钩子的插件，传入 `ProjectConfig` 引用
3. WHEN `post_generate` 钩子被触发, THE PluginManager SHALL 按插件优先级顺序调用所有注册了该钩子的插件，传入项目目录路径和 `ProjectConfig` 引用
4. WHEN `modify_context` 钩子被触发, THE PluginManager SHALL 允许插件向 TemplateContext 添加自定义变量（不允许删除或修改内置变量）
5. WHEN `modify_templates` 钩子被触发, THE PluginManager SHALL 允许插件向模板集合添加新模板或替换现有模板
6. THE PluginAPI SHALL 为每个钩子点定义 Rust trait，插件通过实现 trait 来注册钩子处理函数
7. WHEN 多个插件注册了同一个钩子点, THE PluginManager SHALL 按照插件清单中声明的 `priority` 字段（默认值 100，数值越小优先级越高）顺序执行
8. WHEN 钩子执行过程中插件返回错误, THE PluginManager SHALL 记录错误信息并继续执行后续插件（不中断生成流程），同时在最终报告中汇总所有插件错误
9. THE PluginAPI SHALL 提供 `PluginContext` 结构体，包含当前 CLI_Tool 版本、项目配置、插件自身配置等信息，作为所有钩子函数的输入参数

### 需求 5：插件提供的模板集 / Plugin-Provided Template Sets

**用户故事 / User Story:** 作为一名插件开发者，我希望插件能够提供额外的模板文件集，以便扩展生成项目的文件结构。

As a plugin developer, I want plugins to provide additional template file sets, so that the generated project's file structure can be extended.

#### 验收标准 / Acceptance Criteria

1. WHEN 插件声明了 `templates` 能力, THE PluginManager SHALL 从插件目录的 `templates/` 子目录加载模板文件
2. THE TemplateResolver SHALL 在合并模板时按以下优先级处理：用户自定义模板 > 插件模板 > 内置模板
3. WHEN 多个插件提供相同路径的模板文件, THE TemplateResolver SHALL 使用优先级更高的插件的模板（按 `priority` 字段排序）
4. THE PluginManager SHALL 将插件模板传递给 TemplateResolver，使其参与模板继承系统（支持 `extends` 和 `block`/`override`）
5. WHEN 插件模板使用 `extends` 指令引用内置模板, THE TemplateResolver SHALL 正确解析继承关系
6. THE PluginManager SHALL 验证插件模板文件使用有效的 Handlebars 语法

### 需求 6：插件自定义命令 / Plugin Custom Commands

**用户故事 / User Story:** 作为一名插件开发者，我希望插件能够注册自定义 CLI 子命令，以便为用户提供插件特有的功能入口。

As a plugin developer, I want plugins to register custom CLI subcommands, so that users can access plugin-specific functionality.

#### 验收标准 / Acceptance Criteria

1. WHEN 插件声明了 `commands` 能力, THE PluginManager SHALL 注册插件定义的子命令到 CLI_Tool 的命令树中
2. THE CLI_Tool SHALL 将插件命令注册在 `axum-app-create plugin run <COMMAND>` 命名空间下，避免与内置命令冲突
3. WHEN 用户执行插件命令, THE CLI_Tool SHALL 将命令参数传递给对应插件的命令处理函数
4. THE PluginAPI SHALL 为自定义命令定义 trait，包含命令名称、描述、参数定义和执行函数
5. WHEN 插件命令执行失败, THE CLI_Tool SHALL 输出包含插件名称和命令名称的错误信息

### 需求 7：插件配置 / Plugin Configuration

**用户故事 / User Story:** 作为一名开发者，我希望能够为每个插件提供配置参数，以便根据项目需求定制插件行为。

As a developer, I want to provide configuration parameters for each plugin, so that I can customize plugin behavior based on project needs.

#### 验收标准 / Acceptance Criteria

1. THE UserConfig SHALL 支持 `[plugins.<plugin_name>]` 段，允许用户为每个插件设置配置参数
2. THE PluginManifest SHALL 支持 `[config]` 段，定义插件接受的配置参数及其默认值和类型
3. WHEN 加载插件配置, THE PluginManager SHALL 合并清单中的默认值与用户配置文件中的覆盖值（用户配置优先）
4. WHEN 用户配置中包含插件清单未定义的配置键, THE PluginManager SHALL 输出警告信息并忽略未知配置键
5. THE PluginManager SHALL 将合并后的配置作为 `toml::Value` 传递给插件的钩子函数

### 需求 8：插件依赖解析 / Plugin Dependency Resolution

**用户故事 / User Story:** 作为一名插件开发者，我希望插件能够声明对其他插件的依赖，以便构建可组合的插件生态。

As a plugin developer, I want plugins to declare dependencies on other plugins, so that a composable plugin ecosystem can be built.

#### 验收标准 / Acceptance Criteria

1. WHEN 安装插件时, THE PluginManager SHALL 解析插件的依赖关系并验证所有依赖已安装且版本兼容
2. WHEN 插件依赖未安装, THE PluginManager SHALL 输出缺失依赖的名称和版本要求
3. WHEN 插件依赖存在循环引用, THE PluginManager SHALL 检测循环并输出包含循环路径的错误信息
4. THE PluginManager SHALL 按照依赖拓扑顺序加载插件（被依赖的插件先加载）
5. WHEN 依赖版本不兼容, THE PluginManager SHALL 输出版本冲突的详细信息（插件名称、要求版本、实际版本）

### 需求 9：CLI 参数扩展 / CLI Arguments Extension

**用户故事 / User Story:** 作为一名开发者，我希望新的插件管理功能通过清晰的 CLI 参数和子命令暴露，以便方便地管理和使用插件。

As a developer, I want new plugin management features exposed through clear CLI arguments and subcommands, so that I can conveniently manage and use plugins.

#### 验收标准 / Acceptance Criteria

1. THE CLI_Tool SHALL 新增 `plugin` 子命令组，包含 `install`、`uninstall`、`enable`、`disable`、`list`、`info`、`run` 子命令
2. THE `plugin install` 子命令 SHALL 接受 `<SOURCE>` 参数，支持本地路径、Git URL 和 crate 名称三种格式
3. THE `plugin install` 子命令 SHALL 支持 `--git <URL>` 标志显式指定 Git 来源
4. THE `plugin install` 子命令 SHALL 支持 `--path <DIR>` 标志显式指定本地路径来源
5. WHEN 使用 `--help` 标志, THE CLI_Tool SHALL 显示所有插件相关命令的双语帮助信息（英文 + 中文）
6. THE CLI_Tool SHALL 将版本号更新为 `0.4.0`
7. THE `new` 子命令 SHALL 新增 `--plugin <NAME>` 标志，允许在生成时指定额外启用的插件

### 需求 10：向后兼容性 / Backward Compatibility

**用户故事 / User Story:** 作为一名现有用户，我希望升级到 v0.4.0 后现有的使用方式不受影响，以便平滑过渡到新版本。

As an existing user, I want upgrading to v0.4.0 to not break my existing workflows, so that I can smoothly transition to the new version.

#### 验收标准 / Acceptance Criteria

1. WHEN 未安装任何插件, THE CLI_Tool SHALL 使用全部内置功能生成项目（与 v0.3.0 行为一致）
2. WHEN 使用 v0.3.0 的 CLI 参数组合, THE CLI_Tool SHALL 生成与 v0.3.0 功能一致的项目
3. THE Generator SHALL 确保所有现有测试继续通过
4. WHEN 插件系统初始化失败（如缓存目录不可写）, THE CLI_Tool SHALL 输出警告并以无插件模式继续运行

### 需求 11：插件安全与信任模型 / Plugin Security and Trust Model

**用户故事 / User Story:** 作为一名开发者，我希望插件系统提供基本的安全保障，以便我能信任并安全地使用第三方插件。

As a developer, I want the plugin system to provide basic security guarantees, so that I can trust and safely use third-party plugins.

#### 验收标准 / Acceptance Criteria

1. WHEN 首次安装来自 Git 或 crates.io 的插件, THE CLI_Tool SHALL 显示插件的元数据摘要并要求用户确认安装
2. THE PluginManager SHALL 限制插件的文件系统访问范围：插件只能读写项目目录和插件自身的缓存目录
3. WHEN 插件尝试访问受限路径, THE PluginManager SHALL 拒绝操作并记录安全违规日志
4. THE PluginManifest SHALL 支持 `[permissions]` 段，声明插件需要的权限（如 `network`、`filesystem`、`exec`）
5. WHEN 插件请求的权限超出用户授权范围, THE PluginManager SHALL 在安装时提示用户确认权限授予
