# 补丁历史与安全卸载 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为当前项目增加持久化的补丁历史页，并允许安全卸载由 GameTranslator 安装的 Ren'Py 翻译补丁。

**Architecture:** 桌面端把补丁导出、安装信息写入应用数据目录的 JSON 历史库；历史库以项目绝对路径筛选。卸载时重读该历史项的补丁清单：有备份则恢复，无备份仅在当前文件哈希仍等于已安装补丁时才删除，任何不匹配都停止操作。

**Tech Stack:** Tauri 2、Rust、React 19、TypeScript、Vitest、Cargo test。

---

### Task 1: 定义历史记录与安全卸载核心

**Files:**
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Test: `apps/desktop/src-tauri/src/lib.rs`

1. 先写失败测试：导出/安装记录可按项目读取；卸载会恢复备份；无备份时仅删除哈希匹配的安装文件；哈希不匹配时拒绝卸载。
2. 运行 `cargo test -p game-translator-desktop patch_history`，确认测试失败。
3. 定义 `PatchHistoryEntry`、JSON 文件读写函数和按项目筛选函数。记录保存到 Tauri `app_data_dir` 下的 `patch-history.json`。
4. 导出成功后写入“已导出”记录；安装成功后更新同一记录为“已安装”。
5. 实现卸载：验证历史条目、清单、相对路径；恢复补丁目录的 `backup/` 文件，或仅在目标文件哈希与清单目标哈希一致时删除；成功后清除安装状态。
6. 运行测试，确认通过。

### Task 2: 暴露用例级 Tauri Command

**Files:**
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Test: `apps/desktop/src-tauri/src/lib.rs`

1. 写失败测试，确认 `list_patch_history` 只返回当前项目的记录，`uninstall_translation_patch` 调用安全卸载。
2. 运行对应 Rust 测试，确认失败。
3. 添加 `list_patch_history`、`uninstall_translation_patch`，并注册到 `invoke_handler`。
4. 运行桌面 Rust 测试，确认通过。

### Task 3: 增加侧栏历史页

**Files:**
- Create: `apps/desktop/src/features/history/PatchHistory.tsx`
- Modify: `apps/desktop/src/App.tsx`
- Modify: `apps/desktop/src/styles/global.css`
- Modify: `apps/desktop/src/test/project-flow.test.tsx`

1. 写失败前端测试：左侧出现“历史”入口；打开后请求当前项目历史；已安装补丁可点击卸载并从列表更新。
2. 运行 `npm test -- --run src/test/project-flow.test.tsx`，确认失败。
3. 实现历史列表、导出/安装状态、空态、卸载中状态和错误反馈。
4. 在 App 中加载历史，并在导出、安装、卸载后刷新列表。
5. 运行前端测试，确认通过。

### Task 4: 完整验证

1. 运行 `cargo test -p game-translator-desktop`。
2. 运行 `npm --prefix apps/desktop test`。
3. 运行 `npm --prefix apps/desktop run typecheck` 和 `npm --prefix apps/desktop run build`。
4. 运行 `git diff --check`，确认无空白错误。
