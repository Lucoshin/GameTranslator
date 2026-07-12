# 内容来源适配器 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 将翻译、QA 与模型编排从游戏引擎细节中解耦，使新内容类型可以以明确、可测试且安全的适配器接入。

**Architecture:** 新增内容来源契约与输出契约；翻译工作台继续只接收稳定 Segment。RPG Maker / Ren'Py 先通过薄包装兼容新契约，再在真实样本测试后增加模组、文档和字幕适配器。

**Tech Stack:** Rust workspace、Tauri 2、React、TypeScript、Cargo test、Vitest。

---

### Task 1: 定义内容来源与输出契约

**Files:**
- Create: `crates/content-core/Cargo.toml`
- Create: `crates/content-core/src/lib.rs`
- Create: `crates/content-core/src/source.rs`
- Create: `crates/content-core/src/output.rs`
- Test: `crates/content-core/tests/contract.rs`

1. 写失败测试：来源适配器必须声明格式标识、内容类别、只读扫描能力和允许的输出能力。
2. 定义 `ContentSourceAdapter`，输出稳定 Segment 与结构化来源元数据。
3. 定义 `OutputAdapter`，明确仅可导出、可安装、可卸载三种能力；不得默认允许安装。
4. 运行契约测试。

本轮只完成任务 1，并将通用 `Segment` 的所有权迁移到 `content-core`；`engine-core` 仅保留向后兼容的再导出和现有游戏引擎契约。这样不会中断已上线的 Ren'Py / RPG Maker 调用路径，也不会把 RimWorld 预先标记为已支持。

进度：`content-core`、`content-game` 和 `app-core` 的来源注册入口已完成并有契约测试；RimWorld 已完成 `Keyed` / `DefInjected` 提取、中文独立语言包导出、桌面端目录选择/翻译/导出、安装、卸载和历史记录接入。端到端测试覆盖导出、安装和卸载。后续应以真实 Workshop 模组扩展复杂 XML、`Strings`、`Backstories` 与多语言输出。

### Task 2: 包装现有游戏适配器

**Files:**
- Modify: `crates/engine-rpgmaker/src/lib.rs`
- Modify: `crates/engine-renpy/src/lib.rs`
- Create: `crates/content-game/Cargo.toml`
- Create: `crates/content-game/src/lib.rs`
- Test: `crates/content-game/tests/rpgmaker.rs`
- Test: `crates/content-game/tests/renpy.rs`

1. 为现有 RPG Maker 与 Ren'Py fixture 写失败测试，验证新契约返回的片段 ID、原文、上下文和来源文件与旧接口一致。
2. 用薄包装实现 `ContentSourceAdapter`，不复制已有提取逻辑。
3. 仅为已验证的 Ren'Py 输出声明安装/卸载能力；RPG Maker 保持导出能力。
4. 运行旧测试和新契约测试，确保补丁格式与现有命令不变。

### Task 3: 改造应用服务与 UI 用语

**Files:**
- Modify: `crates/app-core/src/engine.rs`
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Modify: `apps/desktop/src/features/projects/ProjectHome.tsx`
- Modify: `apps/desktop/src/features/projects/ProjectOverview.tsx`
- Test: `apps/desktop/src/test/project-flow.test.tsx`

1. 写失败测试：选择内容来源后，项目概览只显示适配器声明的预览、导出、安装和卸载操作。
2. 将桌面端用例级命令依赖新来源契约。
3. 将界面术语从“游戏目录”逐步调整为“内容来源”，但在现有游戏工作流中保留明确的游戏说明。
4. 验证没有适配器宣称支持的内容类型不会出现在可选列表中。

### Task 4: 在真实样本基础上增加模组适配器

**Prerequisite:** 用户提供或确认可公开测试的星露谷 / RimWorld 模组样本和目标游戏版本。

1. 先读取官方格式或真实资源，记录字段、语言包位置、安装路径和回滚行为。
2. 每个模组生态单独建立 fixture 与检测、提取、导出、安装测试。
3. 未确认格式时返回显式“不支持”，不得猜测 JSON、XML 或目录约定。

### Task 5: 增加非游戏内容类型

1. 文档从 Markdown、TXT、EPUB 中选择一个最小格式，输出翻译副本。
2. 视频仅从已有 SRT/VTT 字幕开始；转写、OCR、烧录字幕属于后续独立能力。
3. 每种来源都必须保持原文件不变，并由输出适配器声明可回滚边界。

## 验证

```powershell
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm --prefix apps/desktop run typecheck
npm --prefix apps/desktop test
```

验收条件：现有游戏功能与补丁格式无回归；未实现的内容类型不会被标记为“支持”；每个新来源都有真实 fixture 和输出安全测试。
