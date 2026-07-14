# Unified Project Studio Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 将游戏与书籍流程收敛到同一项目中心、应用外壳和阶段导航，同时保留各自真实处理链路。

**Architecture:** 提取可复用的 `WorkspaceChrome` 负责常驻侧栏与顶部栏；`App` 只管理项目类型和当前阶段。项目中心成为默认路由，游戏与书籍分别向统一外壳提供内容区域，不再各自渲染完整应用框架。

**Tech Stack:** React 19、TypeScript、CSS、Vitest、Testing Library、Tauri 2

---

### Task 1: 锁定统一导航行为

**Files:**
- Modify: `apps/desktop/src/test/book-workspace.test.tsx`
- Modify: `apps/desktop/src/test/project-flow.test.tsx`

1. 增加“启动即项目中心”的测试。
2. 增加游戏与书籍打开后均存在 `应用导航`，且含“项目、概览、翻译、校对、导出”的测试。
3. 增加书籍工作台不再存在旧 `书籍项目导航` 的测试。
4. 运行相关测试并确认因旧分叉结构而失败。

### Task 2: 提取统一工作区外壳

**Files:**
- Create: `apps/desktop/src/features/workspace/WorkspaceChrome.tsx`
- Create: `apps/desktop/src/features/workspace/workspace-chrome.css`
- Modify: `apps/desktop/src/App.tsx`

1. 实现统一侧栏、项目顶栏和内容插槽。
2. 将游戏项目现有内联 rail/topbar 移入统一组件。
3. 将品牌按钮语义统一为“返回项目中心”。
4. 运行测试并确认游戏项目行为恢复。

### Task 3: 将项目中心设为默认入口

**Files:**
- Modify: `apps/desktop/src/App.tsx`
- Modify: `apps/desktop/src/features/books/ProjectCenter.tsx`
- Modify: `apps/desktop/src/features/books/book-workspace.css`

1. 默认视图改为项目中心。
2. 项目中心嵌入统一外壳，移除独立 library rail。
3. 游戏卡片直接触发目录选择，书籍按钮直接触发文件选择。
4. 保留最近游戏项目自动恢复，但不跳过项目中心；在项目中心显示“继续游戏项目”。
5. 运行项目流程测试。

### Task 4: 将书籍编辑器嵌入统一外壳

**Files:**
- Modify: `apps/desktop/src/features/books/BookWorkspace.tsx`
- Modify: `apps/desktop/src/features/books/book-workspace.css`
- Modify: `apps/desktop/src/App.tsx`

1. 删除书籍独立 header、nav 和重复模型按钮。
2. 接入统一阶段：概览显示项目/语言摘要，翻译进入书稿并触发按章任务，校对显示同一编辑器的逐段模式，导出提供单一导出动作。
3. 保留章节树、连续书稿、检查器、自动保存和 Ctrl+Enter。
4. 运行书籍交互测试。

### Task 5: 回归与可见验收

**Files:**
- Modify: `apps/desktop/src/styles/global.css`
- Modify: `apps/desktop/src/features/books/book-workspace.css`

1. 搜索并清除旧 `homeOpen`、`ProjectHome`、`library-rail` 与 `book-nav` 调用路径。
2. 运行 `npm test`、`npm run build`。
3. 运行 Rust 书籍与桌面测试、Clippy。
4. 使用 `custom-protocol` 构建 release EXE。
5. 启动 EXE，在 1280px 下分别打开游戏入口与真实 TXT 书籍，确认导航位置不变化。
