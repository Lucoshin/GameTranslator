# GameTranslator Architecture Convergence Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 将桌面后端中的业务编排收口到可复用应用层，统一内容适配器和持久化边界，并保持现有前端协议与用户行为不变。

**Architecture:** `app-core` 成为扫描、翻译、QA、补丁和任务恢复的唯一应用用例层；Tauri 仅负责系统对话框、系统路径、事件桥接和 DTO 序列化。内容格式通过静态注册表选择，任务与补丁历史统一写入 SQLite，应用错误以稳定代码跨越前后端边界。

**Tech Stack:** Rust 2021、Tauri 2、SQLite/rusqlite、React 19、TypeScript、Vitest、Cargo test。

---

### Task 1: 固化应用契约与结构化错误

**Files:**
- Create: `crates/app-core/src/error.rs`
- Create: `crates/app-core/src/models.rs`
- Create: `crates/app-core/tests/contracts.rs`
- Modify: `crates/app-core/src/lib.rs`

1. 先写失败测试，约束错误代码、任务状态和现有 camelCase DTO 所需字段语义。
2. 运行 `cargo test -p game-translator-app-core --test contracts`，确认因类型缺失失败。
3. 实现 `AppError`、`AppErrorCode`、`TaskStatus`、语言、扫描、翻译和补丁领域模型。
4. 再运行测试确认通过。

### Task 2: 统一静态内容适配器注册表

**Files:**
- Create: `crates/app-core/src/adapters.rs`
- Create: `crates/app-core/tests/adapter_registry.rs`
- Modify: `crates/app-core/src/content.rs`
- Modify: `crates/app-core/src/engine.rs`

1. 先写失败测试，要求同一个注册表完成 RPG Maker、Ren'Py、RimWorld 检测与提取。
2. 实现静态 `AdapterRegistry`，集中保存格式 ID、检测、提取和输出能力。
3. 让旧游戏入口委托给新注册表，保留兼容 API，但删除重复选择逻辑。
4. 全局检索旧的格式 `match` 和重复适配器数组。

### Task 3: 完整任务持久化

**Files:**
- Create: `crates/project-store/src/tasks.rs`
- Create: `crates/project-store/tests/tasks.rs`
- Modify: `crates/project-store/src/migrations.rs`
- Modify: `crates/project-store/src/lib.rs`

1. 先写失败测试，覆盖任务创建、进度更新、翻译结果/QA 写入、重开数据库恢复和显式状态转换。
2. 扩展 SQLite schema，保存任务元数据、状态、进度、失败项与序列化结果。
3. 使用事务更新任务快照；非法状态转换返回明确错误。
4. 运行 project-store 全部测试，验证旧数据库迁移兼容。

### Task 4: 提取应用服务

**Files:**
- Create: `crates/app-core/src/provider_factory.rs`
- Create: `crates/app-core/src/translation_service.rs`
- Create: `crates/app-core/src/patch_service.rs`
- Create: `crates/app-core/src/settings_service.rs`
- Create: `crates/app-core/tests/translation_service.rs`
- Modify: `crates/app-core/Cargo.toml`
- Modify: `crates/app-core/src/lib.rs`

1. 先以 mock Provider 写失败测试，覆盖扫描→缓存→翻译→QA→任务持久化→恢复。
2. 将性能配置下沉到 `translation_service`，Provider 构造下沉到 `provider_factory`。
3. 将补丁历史、安装/卸载和设置持久化从 Tauri 文件迁入应用服务/SQLite。
4. 保持 API Key 只由凭据仓储读取，不进入任务快照或前端响应。

### Task 5: 瘦身并拆分 Tauri 接口层

**Files:**
- Create: `apps/desktop/src-tauri/src/commands/mod.rs`
- Create: `apps/desktop/src-tauri/src/commands/project.rs`
- Create: `apps/desktop/src-tauri/src/commands/translation.rs`
- Create: `apps/desktop/src-tauri/src/commands/patch.rs`
- Create: `apps/desktop/src-tauri/src/commands/settings.rs`
- Create: `apps/desktop/src-tauri/src/dto.rs`
- Create: `apps/desktop/src-tauri/src/events.rs`
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Modify: `apps/desktop/src-tauri/Cargo.toml`

1. 添加契约测试，锁定现有 Command 名称、请求字段和响应字段。
2. 将 DTO、事件和四类 Command 拆入独立模块。
3. Command 只获取 Tauri 系统资源、调用 `app-core`、映射结构化错误。
4. 删除 Tauri 层中的翻译、QA、哈希、SQLite 和补丁业务 helper。
5. 全局检索旧 helper、旧 DTO 和直接底层 crate 依赖，确认无残留。

### Task 6: 前端结构化错误与恢复入口

**Files:**
- Modify: `apps/desktop/src/App.tsx`
- Create: `apps/desktop/src/api/contracts.ts`
- Create: `apps/desktop/src/api/errors.ts`
- Modify: `apps/desktop/src/test/project-flow.test.tsx`

1. 先写失败测试，验证恢复未完成任务及不同错误代码的稳定展示。
2. 集中定义前端 Command 契约和错误映射。
3. 启动时读取可恢复任务；用户确认后恢复未完成批次。
4. 保留现有字段名与界面流程。

### Task 7: 文档、废弃清理与完整验证

**Files:**
- Create: `docs/decisions/ADR-003-application-service-boundary.md`
- Modify: `README.md`
- Modify: `docs/plans/2026-07-12-desktop-state-persistence.md`

1. 记录应用服务边界、静态适配器注册、SQLite 任务快照和结构化错误决策。
2. 删除废弃入口、旧 JSON 历史文件逻辑、无调用符号和不再需要的直接依赖。
3. 运行 `cargo fmt --check`、Clippy、全部 Rust 测试、前端测试、类型检查和生产构建。
4. 核对 `git diff --check`、旧符号检索及 Tauri Command 契约测试。
