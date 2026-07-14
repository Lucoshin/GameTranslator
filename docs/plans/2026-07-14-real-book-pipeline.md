# Real Book Translation Pipeline Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为桌面应用提供 TXT、Markdown、EPUB、DOCX 的真实导入、项目恢复、模型翻译、人工校对保存与独立导出。

**Architecture:** 新建 `content-book` Rust crate 负责格式解析和统一领域模型。Tauri 命令负责文件选择、应用数据目录持久化、调用现有翻译编排器与导出；React 只消费 `BookProject` 契约并提交用户操作。

**Tech Stack:** Rust、serde、quick-xml、zip、encoding_rs、Tauri 2、React 19、Vitest。

---

### Task 1: 书籍领域模型与纯文本适配器

**Files:**
- Create: `crates/content-book/Cargo.toml`
- Create: `crates/content-book/src/lib.rs`
- Create: `crates/content-book/tests/plain_text.rs`
- Modify: `Cargo.toml`

1. 写失败测试：TXT 按章标题和空行拆分，Markdown 按标题拆分。
2. 定义 `BookProject`、`BookChapter`、`BookSegment`、`SegmentStatus`。
3. 实现 UTF-8/UTF-16/GB18030 文本解码和稳定段落 ID。
4. 运行 `cargo test -p game-translator-content-book`。

### Task 2: EPUB 与 DOCX 适配器

**Files:**
- Modify: `crates/content-book/src/lib.rs`
- Create: `crates/content-book/tests/containers.rs`

1. 用内存 ZIP fixture 写失败测试。
2. EPUB 按 OPF spine 顺序读取 XHTML；DOCX 按 Word 段落与标题样式分章。
3. 对损坏容器和空书稿返回可读错误。
4. 运行 crate 测试。

### Task 3: Tauri 项目命令与模型翻译

**Files:**
- Modify: `apps/desktop/src-tauri/Cargo.toml`
- Modify: `apps/desktop/src-tauri/src/commands/dto.rs`
- Modify: `apps/desktop/src-tauri/src/commands/mod.rs`

1. 写失败测试覆盖项目保存/读取和 Markdown 导出。
2. 添加 `import_book_project`、`list_book_projects`、`save_book_project`、`translate_book_project`、`export_book_project`。
3. 翻译复用 `TranslationOrchestrator`，译文状态写为 `draft`，QA 异常写为 `issue`。
4. 注册命令并运行桌面 Rust 测试。

### Task 4: 前端真实项目接入

**Files:**
- Modify: `apps/desktop/src/App.tsx`
- Modify: `apps/desktop/src/features/books/ProjectCenter.tsx`
- Modify: `apps/desktop/src/features/books/BookWorkspace.tsx`
- Modify: `apps/desktop/src/test/book-workspace.test.tsx`

1. 写失败测试：导入返回的真实书稿出现在项目中心并进入工作台。
2. 把工作台样例状态替换为 `BookProject` props，编辑时调用保存命令。
3. 接通翻译本章和 Markdown 导出，显示加载/错误/完成反馈。
4. 运行前端全量测试。

### Task 5: 构建与端到端验收

1. 运行 `cargo test --workspace`、`npm test -- --run` 和 `npm run build`。
2. 生产构建 EXE。
3. 使用 TXT、Markdown、EPUB、DOCX fixture 各导入一次，验证章节、翻译保存和导出。
