# GameTranslator

GameTranslator 是一个本地优先、用户自带模型密钥的开源游戏汉化工作台。当前 MVP 面向未加密的 RPG Maker MV/MZ 项目，提供文本提取、上下文批量翻译、控制码保护、质量检查和安全补丁导出。

> 当前处于开发预览阶段。桌面端已经接通本地目录选择、项目扫描、Provider 翻译、人工校对和补丁导出，但任务状态目前只保存在当前会话；崩溃恢复、SQLite 缓存和大项目后台进度尚未串入桌面流程。

## 当前能力

- 识别 RPG Maker MV/MZ 数据目录。
- 提取地图事件、公共事件、选项、滚动文本及数据库名称和说明。
- 保护 `\V[n]`、`\N[n]`、`\C[n]`、`\I[n]` 控制码。
- 接入 OpenAI-compatible 和 Ollama 结构化翻译接口。
- 支持场景分批、缓存、暂停、限流重试和失败拆批。
- 使用 SQLite 保存项目、任务、翻译缓存、术语和翻译记忆。
- 在独立目录回写翻译，校验 SHA-256 后生成补丁清单。
- 使用 Windows Credential Manager 保存 API Key，不写入项目配置。
- 从桌面界面完成“选择目录、配置模型、翻译、校对、导出补丁”。

## 不支持

- 加密资源解包或绕过游戏保护。
- Unity、Ren'Py、Wolf RPG 等其他引擎。
- 插件脚本动态文本、图片文字、OCR、字体替换、语音和配音。
- 云端账号、托管 API、多用户协作和汉化包分发。

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

## 架构

```text
React UI -> Tauri Commands -> Application Services
                                  |-- RPG Maker Adapter
                                  |-- Translation Orchestrator
                                  |-- Provider API
                                  |-- QA Engine
                                  |-- SQLite Store
                                  `-- Patch Builder
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
