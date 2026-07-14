# GameTranslator

GameTranslator 是一个本地优先、用户自带模型密钥的游戏与书籍翻译工作台。当前 MVP 面向 RPG Maker MV/MZ、Ren'Py 和常见书稿文件，提供文本提取、多语言批量翻译、质量检查、人工校对与独立交付。

> 当前处于开发预览阶段。桌面端已经接通本地目录选择、项目扫描、并发 Provider 翻译、实时批次与 QA 进度、SQLite 翻译缓存和可恢复任务快照、人工校对及补丁导出。

## 当前能力

- 识别 RPG Maker MV/MZ 数据目录。
- 识别 Ren'Py 发行版并通过游戏自带运行时生成标准翻译模板。
- 缓存未变化的 Ren'Py 模板，避免扫描、翻译和导出阶段重复启动游戏运行时。
- 提取地图事件、公共事件、选项、滚动文本及数据库名称和说明。
- 保护 `\V[n]`、`\N[n]`、`\C[n]`、`\I[n]` 控制码。
- 接入 OpenAI-compatible 和 Ollama 结构化翻译接口。
- 支持字符预算动态分批、1–16 路并发、实时进度、限流重试和失败拆批。
- 使用 SQLite 持久化通过 QA 的精确翻译缓存，模型、语言、Provider 或提示词变化会自动失效。
- 持久化未完成任务的项目、进度和运行快照；重启后可重新扫描项目并从精确缓存继续。
- 在独立目录回写翻译，校验 SHA-256 后生成补丁清单。
- 使用 Windows Credential Manager 保存 API Key，不写入项目配置。
- 从桌面界面完成“选择目录、配置模型、翻译、校对、导出补丁”。
- 源语言可自动检测或手动指定，目标语言支持常用预设和自定义 BCP 47 语言代码。
- 导入 TXT、Markdown、EPUB、DOCX 书稿，在独立三栏工作台中逐段翻译与校对。
- 将书籍译稿导出为 Markdown、可编辑 DOCX、EPUB 3 或嵌入中文字体的 PDF，并持久化真实导出历史。
- PDF 支持大32开、A5、16开成品尺寸、页码和章节另起页；DOCX/EPUB 写入常用出版元数据。

## 不支持

- 加密资源解包或绕过游戏保护。
- Unity、Wolf RPG 等其他引擎。
- 插件脚本动态文本、图片文字、OCR、字体替换、语音和配音。
- 各语言的游戏字体覆盖、RTL 从右向左排版和全部语言组合的专项 QA 保证。
- 云端账号、托管 API、多用户协作和翻译包分发。

完整范围见 [支持矩阵](docs/supported-games.md)。

## 开发环境

- Windows 10/11
- Rust stable，包含 `rustfmt` 和 `clippy`
- Node.js 24 与 npm
- WebView2 Runtime

```powershell
rustup component add rustfmt clippy
cd apps/desktop
npm install
npm run dev
```

Rust 核心测试：

```powershell
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

前端验证：

```powershell
npm --prefix apps/desktop test
npm --prefix apps/desktop run typecheck
npm --prefix apps/desktop run build
```

生成可独立运行、内嵌前端资源的 Windows EXE：

```powershell
npm --prefix apps/desktop run build
cargo build -p game-translator-desktop --release --features custom-protocol
```

`custom-protocol` 不可省略；否则 Tauri 会按开发模式访问本机 Vite 服务。

## 架构

```text
React UI -> Tauri Commands -> app-core Application Services
                                  |-- RPG Maker Adapter
                                  |-- Translation Orchestrator
                                  |-- Provider API
                                  |-- QA Engine
                                  |-- SQLite Store
                                  |-- Patch Builder
                                  `-- Book Publication Writers
```

详细设计见 [产品设计](docs/plans/2026-07-11-game-translator-design.md)，技术选型见 [ADR-001](docs/decisions/ADR-001-desktop-architecture.md)。

## 安全与隐私

- 原游戏在扫描、提取和翻译阶段保持只读。
- API Key 通过系统凭据库保存，不进入 SQLite、日志或补丁。
- 模型请求会把待翻译游戏文本发送给用户配置的提供商。
- 补丁导出前重新验证源文件哈希，版本不一致时停止。

安全问题请按 [SECURITY.md](SECURITY.md) 私下报告。

## 许可证

[MIT](LICENSE)。本项目许可证不授予用户重新分发第三方商业游戏资源的权利。
