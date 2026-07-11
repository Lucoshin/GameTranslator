# GameTranslator MVP Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 构建一个 Windows 优先的桌面应用，为未加密 RPG Maker MV/MZ 游戏完成可恢复的一键简体中文汉化和补丁导出。

**Architecture:** 使用 Rust workspace 承载引擎适配、翻译编排、模型提供商、QA 和 SQLite 存储，通过用例级 Tauri Command 暴露给 React 界面。所有操作在独立工作目录完成，原游戏始终只读。

**Tech Stack:** Tauri 2、Rust stable、React、TypeScript、Vite、SQLite、Vitest、Playwright、Cargo test。

---

## 实施约束

- 开始实现前初始化 Git，并创建独立 worktree。
- 每项行为先写失败测试，再做最小实现。
- 不实现加密解包、Unity、动态插件、OCR、图片翻译或云端账号。
- 不直接覆盖原游戏，不明文保存 API Key。
- 每个任务完成后运行该任务测试并提交一个语义清晰的 commit。

### Task 1: 初始化桌面工程与质量门禁

**Files:**
- Create: `Cargo.toml`
- Create: `crates/app-core/Cargo.toml`
- Create: `crates/app-core/src/lib.rs`
- Create: `apps/desktop/package.json`
- Create: `apps/desktop/src-tauri/Cargo.toml`
- Create: `apps/desktop/src-tauri/src/lib.rs`
- Create: `apps/desktop/src/main.tsx`
- Create: `apps/desktop/src/App.tsx`
- Create: `apps/desktop/vite.config.ts`
- Create: `apps/desktop/vitest.config.ts`
- Create: `rustfmt.toml`
- Create: `.gitignore`
- Create: `.github/workflows/ci.yml`

**Steps:**

1. 初始化 Git，并创建功能 worktree。
2. 创建最小 Rust workspace 和 Tauri React 应用。
3. 添加一个会失败的 Rust smoke test 和一个会失败的 React smoke test。
4. 运行 `cargo test --workspace` 与 `npm test`，确认测试因缺少实现失败。
5. 实现最小应用壳，使两组 smoke test 通过。
6. 配置 `cargo fmt --check`、`cargo clippy --workspace --all-targets -- -D warnings`、TypeScript 检查和 Vitest。
7. 配置 GitHub Actions 在 Windows 上运行全部质量门禁。
8. 提交 `chore: initialize tauri workspace`。

### Task 2: 定义引擎适配契约与项目扫描

**Files:**
- Create: `crates/engine-core/Cargo.toml`
- Create: `crates/engine-core/src/lib.rs`
- Create: `crates/engine-core/src/project.rs`
- Create: `crates/engine-core/src/segment.rs`
- Create: `crates/engine-core/src/adapter.rs`
- Create: `crates/engine-core/tests/adapter_contract.rs`
- Create: `crates/engine-rpgmaker/Cargo.toml`
- Create: `crates/engine-rpgmaker/src/lib.rs`
- Create: `crates/engine-rpgmaker/src/detect.rs`
- Create: `crates/engine-rpgmaker/tests/detect.rs`
- Create: `fixtures/rpgmaker-mv-minimal/www/data/System.json`
- Create: `fixtures/rpgmaker-mz-minimal/data/System.json`
- Create: `fixtures/unsupported/README.txt`

**Steps:**

1. 写测试定义 MV、MZ、不支持目录和缺失文件的预期识别结果。
2. 运行 `cargo test -p engine-rpgmaker detect`，确认失败。
3. 定义 `EngineAdapter`、`DetectedProject`、`Segment` 和显式错误类型。
4. 实现基于目录特征和 JSON 内容的最小 MV/MZ 检测，不添加猜测性降级。
5. 运行适配契约和检测测试，确认全部通过。
6. 提交 `feat: detect supported rpg maker projects`。

### Task 3: 提取 RPG Maker 文本并生成稳定 Segment

**Files:**
- Create: `crates/engine-rpgmaker/src/extract.rs`
- Create: `crates/engine-rpgmaker/src/commands.rs`
- Create: `crates/engine-rpgmaker/tests/extract_maps.rs`
- Create: `crates/engine-rpgmaker/tests/extract_database.rs`
- Create: `fixtures/rpgmaker-mz-dialogue/data/Map001.json`
- Create: `fixtures/rpgmaker-mz-dialogue/data/CommonEvents.json`
- Create: `fixtures/rpgmaker-mz-dialogue/data/Items.json`
- Create: `fixtures/rpgmaker-mz-dialogue/data/Skills.json`

**Steps:**

1. 为地图对话、选项、滚动文本、公共事件和数据库字段编写失败测试。
2. 为脚本、资源路径、空白和纯数字过滤编写失败测试。
3. 运行 `cargo test -p engine-rpgmaker extract`，确认失败。
4. 实现明确字段白名单与事件指令解析。
5. 使用结构位置生成稳定 Segment ID，并附带来源与相邻上下文。
6. 运行提取测试并检查固定快照。
7. 提交 `feat: extract translatable rpg maker segments`。

### Task 4: 实现控制码保护与 QA 核心

**Files:**
- Create: `crates/qa-core/Cargo.toml`
- Create: `crates/qa-core/src/lib.rs`
- Create: `crates/qa-core/src/placeholders.rs`
- Create: `crates/qa-core/src/checks.rs`
- Create: `crates/qa-core/tests/placeholders.rs`
- Create: `crates/qa-core/tests/checks.rs`

**Steps:**

1. 为 `\V[n]`、`\N[n]`、`\C[n]`、`\I[n]` 和混合文本编写往返失败测试。
2. 为缺失、增加、类型改变、非法顺序和未还原占位符编写失败测试。
3. 运行 `cargo test -p qa-core`，确认失败。
4. 实现可逆占位符映射和结构化 QA Finding。
5. 实现阻断错误与普通警告分级。
6. 运行 QA 测试并确认全部通过。
7. 提交 `feat: protect control codes and validate translations`。

### Task 5: 建立 SQLite 项目存储与恢复机制

**Files:**
- Create: `crates/project-store/Cargo.toml`
- Create: `crates/project-store/src/lib.rs`
- Create: `crates/project-store/src/migrations.rs`
- Create: `crates/project-store/src/projects.rs`
- Create: `crates/project-store/src/segments.rs`
- Create: `crates/project-store/src/jobs.rs`
- Create: `crates/project-store/src/glossary.rs`
- Create: `crates/project-store/src/memory.rs`
- Create: `crates/project-store/tests/recovery.rs`
- Create: `crates/project-store/tests/cache.rs`

**Steps:**

1. 为项目、Segment、批次状态、术语和翻译记忆编写仓储失败测试。
2. 为任务中断后只恢复未完成批次编写失败测试。
3. 为相同输入指纹命中缓存、配置变化导致缓存失效编写失败测试。
4. 运行 `cargo test -p project-store`，确认失败。
5. 创建最小迁移和事务边界，实现显式状态转换。
6. 运行存储与恢复测试，确认通过。
7. 提交 `feat: persist translation projects and jobs`。

### Task 6: 实现模型 Provider 与翻译编排

**Files:**
- Create: `crates/provider-core/Cargo.toml`
- Create: `crates/provider-core/src/lib.rs`
- Create: `crates/provider-core/src/openai_compatible.rs`
- Create: `crates/provider-core/src/ollama.rs`
- Create: `crates/provider-core/tests/provider_contract.rs`
- Create: `crates/translation-core/Cargo.toml`
- Create: `crates/translation-core/src/lib.rs`
- Create: `crates/translation-core/src/batch.rs`
- Create: `crates/translation-core/src/orchestrator.rs`
- Create: `crates/translation-core/src/retry.rs`
- Create: `crates/translation-core/tests/orchestration.rs`

**Steps:**

1. 使用本地 mock HTTP server 为连接测试、结构化响应、限流和服务错误编写失败测试。
2. 为按场景分批、暂停、恢复、缓存跳过和批次拆分编写失败测试。
3. 运行 Provider 与编排测试，确认失败且不访问真实网络。
4. 实现统一 Provider trait、OpenAI-compatible 和 Ollama 请求映射。
5. 实现严格响应 ID 校验、有限重试、指数退避和拆批。
6. 将成功结果以事务方式写入存储，失败条目进入人工处理状态。
7. 运行测试并确认没有真实 API 依赖。
8. 提交 `feat: orchestrate resumable llm translation`。

### Task 7: 实现安全回写与补丁导出

**Files:**
- Create: `crates/engine-rpgmaker/src/write.rs`
- Create: `crates/app-core/src/patch.rs`
- Create: `crates/engine-rpgmaker/tests/round_trip.rs`
- Create: `crates/app-core/tests/patch_export.rs`
- Create: `fixtures/rpgmaker-mz-dialogue/expected/Map001.json`

**Steps:**

1. 为“无修改往返语义一致”编写失败测试。
2. 为只修改目标 JSON 路径、保留非目标字段编写失败测试。
3. 为原文件哈希不匹配、阻断 QA 存在和输出 JSON 损坏编写失败测试。
4. 运行回写与补丁测试，确认失败。
5. 实现工作目录回写、重新解析、SHA-256 清单和补丁导出。
6. 验证任何失败都不会修改 fixture 原目录。
7. 运行完整 Rust workspace 测试。
8. 提交 `feat: export verified translation patches`。

### Task 8: 实现桌面端完整任务流

**Files:**
- Create: `apps/desktop/src/features/projects/ProjectHome.tsx`
- Create: `apps/desktop/src/features/projects/ProjectOverview.tsx`
- Create: `apps/desktop/src/features/providers/ProviderDrawer.tsx`
- Create: `apps/desktop/src/features/translation/TranslationProgress.tsx`
- Create: `apps/desktop/src/features/review/SegmentTable.tsx`
- Create: `apps/desktop/src/features/export/ExportPanel.tsx`
- Create: `apps/desktop/src/styles/tokens.css`
- Create: `apps/desktop/src/styles/global.css`
- Create: `apps/desktop/src/test/project-flow.test.tsx`
- Create: `apps/desktop/e2e/project-flow.spec.ts`
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Modify: `apps/desktop/src/App.tsx`

**Steps:**

1. 定义暖灰、墨黑、朱橙强调色、字体、间距和状态色 tokens。
2. 为选择项目、模型连接、开始任务、暂停恢复、校对和导出编写组件失败测试。
3. 实现用例级 Tauri Commands，不向前端暴露任意文件读写能力。
4. 实现单窗口任务流、明确空态、错误态和键盘可访问性。
5. 添加 Playwright 端到端测试，使用 mock Provider 跑通完整流程。
6. 在 1440x900、1280x720 和最小窗口尺寸生成截图并检查溢出与状态可读性。
7. 运行 `npm test`、`npm run typecheck` 和 Playwright。
8. 提交 `feat: add end-to-end desktop translation flow`。

### Task 9: 安全存储、文档与发布验证

**Files:**
- Create: `crates/app-core/src/credentials.rs`
- Create: `crates/app-core/tests/credentials.rs`
- Create: `README.md`
- Create: `CONTRIBUTING.md`
- Create: `SECURITY.md`
- Create: `LICENSE`
- Create: `docs/supported-games.md`
- Create: `docs/patch-format.md`
- Create: `docs/model-providers.md`
- Modify: `.github/workflows/ci.yml`

**Steps:**

1. 为凭据写入、读取、删除和“数据库不含明文密钥”编写失败测试。
2. 接入 Windows Credential Manager，并在测试中使用显式 mock。
3. 编写 README 快速开始、支持矩阵、限制、架构和隐私说明。
4. 记录补丁格式、Provider 配置和贡献适配器的边界。
5. 配置 Windows 安装包构建，但不在 CI 中写入任何签名密钥。
6. 运行 `cargo fmt --check`、Clippy、全部 Rust/前端/E2E 测试和生产构建。
7. 在干净目录安装构建产物，以两个 fixture 完成冒烟验证。
8. 提交 `docs: prepare mvp release`。

## 最终验证

执行：

```powershell
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm --prefix apps/desktop run typecheck
npm --prefix apps/desktop test
npm --prefix apps/desktop run test:e2e
npm --prefix apps/desktop run tauri build
```

预期结果：所有命令退出码为 0；两个未加密 fixture 均能完成扫描、模拟翻译、QA 和补丁导出；不支持项目得到显式错误；原始 fixture 哈希保持不变。

