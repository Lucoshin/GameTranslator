# Desktop State Persistence Implementation Plan

> 2026-07-13 更新：任务进度与恢复快照已统一写入 SQLite；Provider 密钥继续由 Windows Credential Manager 保存。架构依据见 ADR-003。

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 让重新构建或升级后的桌面程序恢复模型配置、最近项目和语言，并可在未选择项目时查看全部补丁历史。

**Architecture:** 后端应用数据目录中的 JSON 文件作为非敏感设置的唯一可信来源，Windows 凭据管理器继续保存 API Key，SQLite 继续保存翻译缓存。前端启动时通过 Tauri command 加载持久化状态，不再依赖 WebView `localStorage`；补丁操作直接使用历史条目携带的项目路径。

**Tech Stack:** Tauri 2、Rust、Serde、React 19、TypeScript、Vitest。

---

### Task 1: 后端桌面偏好持久化

**Files:**
- Modify: `apps/desktop/src-tauri/src/lib.rs`

**Steps:**
1. 先添加失败的 Rust 测试，验证最近项目路径和源/目标语言可以写入并重新读取。
2. 运行指定测试，确认因偏好读写函数不存在而失败。
3. 添加 `DesktopPreferences`、`read_desktop_preferences`、`write_desktop_preferences` 和对应 Tauri commands。
4. 再次运行测试并确认通过。

### Task 2: 全局补丁历史

**Files:**
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Modify: `apps/desktop/src/features/history/PatchHistory.tsx`

**Steps:**
1. 添加失败测试，要求历史列表在不传项目路径时返回全部记录并按时间倒序。
2. 运行测试确认当前仅支持项目过滤。
3. 将 `list_patch_history` 改为可选项目路径，补丁安装、卸载、删除使用条目自身的 `projectPath`。
4. 运行 Rust 测试确认通过。

### Task 3: 前端启动恢复

**Files:**
- Modify: `apps/desktop/src/App.tsx`
- Modify: `apps/desktop/src/test/project-flow.test.tsx`

**Steps:**
1. 添加失败的 Vitest 测试，验证启动时加载后端配置与偏好、自动恢复最近项目、语言设置，并且不读取 `localStorage`。
2. 运行测试确认失败原因是缺少偏好加载 command。
3. 实现启动恢复、选择项目及语言变更时保存偏好，并移除 provider 的 `localStorage` 双写。
4. 增加可见的启动加载错误提示。
5. 运行前端测试确认通过。

### Task 4: 回归验证

**Files:**
- Verify only

**Steps:**
1. 运行 `npm test` 和 `npm run build`。
2. 运行相关 Rust 测试及完整 workspace 测试。
3. 运行 Tauri release `--no-bundle` 构建，确认持久化文件不在构建目录内且现有 AppData 文件未被修改或删除。
4. 检查 diff，清理旧 `localStorage` 依赖和失效的“必须先选项目才能查看历史”提示，保留凭据管理器与 SQLite 翻译缓存。
