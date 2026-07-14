# Publication Export and Book History Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为书籍项目实现 Markdown、DOCX、EPUB 3、大32开/A5/16开 PDF 出版导出，以及真实持久化导出历史。

**Architecture:** `content-book` 负责纯 Rust 格式生成；Tauri 命令处理保存位置、历史持久化和打开文件；React 提供出版信息、格式卡片、版式预设和历史界面。原书始终只读，导出成功后才记录历史。

**Tech Stack:** Rust、serde、zip、quick-xml、PDF writer、Tauri 2、React 19、Vitest。

---

### Task 1: 出版契约与兼容迁移

**Files:**
- Modify: `crates/content-book/src/lib.rs`
- Modify: `crates/content-book/tests/serialization.rs`
- Modify: `apps/desktop/src/features/books/contracts.ts`

1. 写失败测试，要求旧 JSON 缺少出版字段时仍可读取。
2. 添加 `PublicationMetadata`、`BookExportFormat`、`PrintPreset`、`BookExportProfile`、`BookExportRecord`。
3. 为新增字段提供 serde 默认值并同步 TypeScript 契约。
4. 运行 `cargo test -p game-translator-content-book serialization`。

### Task 2: Markdown、DOCX 与 EPUB 3 写入器

**Files:**
- Modify: `crates/content-book/src/lib.rs`
- Create: `crates/content-book/tests/publication_exports.rs`

1. 写失败测试验证 Markdown 元数据、DOCX ZIP 结构/样式和 EPUB mimetype/OPF/nav/spine。
2. 抽取完整书稿段落选择逻辑。
3. 使用现有 ZIP/XML 依赖生成 DOCX 与 EPUB 3。
4. 运行书籍 crate 测试并检查容器内容。

### Task 3: 印刷 PDF 写入器

**Files:**
- Modify: `Cargo.toml`
- Modify: `crates/content-book/Cargo.toml`
- Modify: `crates/content-book/src/lib.rs`
- Modify: `crates/content-book/tests/publication_exports.rs`

1. 写失败测试验证 `%PDF`、MediaBox 尺寸、书名与章节页生成。
2. 加入 PDF 生成依赖并嵌入 Windows 可用中文字体。
3. 实现大32开默认，以及 A5/16开页面尺寸、页边距、装订线和页码。
4. 用 PDF 解析/渲染工具检查输出。

### Task 4: Tauri 导出命令与历史存储

**Files:**
- Modify: `apps/desktop/src-tauri/src/commands/books.rs`
- Modify: `apps/desktop/src-tauri/src/commands/mod.rs`

1. 写失败测试覆盖成功导出后记录、倒序列表和只删除记录。
2. 将 `export_book_project` 改为接受格式与配置并返回记录。
3. 添加 `list_book_export_history`、`delete_book_export_history`、`reexport_book_project`、`open_book_export_location`。
4. 注册命令并运行桌面 Rust 测试。

### Task 5: 出版导出与历史界面

**Files:**
- Modify: `apps/desktop/src/App.tsx`
- Modify: `apps/desktop/src/features/books/BookWorkspace.tsx`
- Modify: `apps/desktop/src/features/books/book-workspace.css`
- Modify: `apps/desktop/src/test/book-workspace.test.tsx`

1. 写失败测试：四种格式按钮始终可见，PDF 默认大32开，导出成功出现在历史。
2. 增加出版信息表单、格式卡片和 PDF 版式选择。
3. 接通导出、再次导出、打开位置和删除历史。
4. 修复导出页颜色变量作用域并展示成功/失败反馈。
5. 运行前端全量测试。

### Task 6: 出版文档质量与发布构建

**Files:**
- Modify: `README.md`
- Modify: `docs/decisions/ADR-004-book-document-project-pipeline.md`

1. 生成包含中文章节的 DOCX、EPUB 和三种 PDF 样例。
2. 解包 EPUB/DOCX 并渲染 DOCX/PDF 页面检查字体、分页、缩进和页码。
3. 运行 `cargo fmt --all -- --check`、书籍 crate 测试、前端全量测试和生产构建。
4. 使用 `custom-protocol` 重建 Release EXE。
