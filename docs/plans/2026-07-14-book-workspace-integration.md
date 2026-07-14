# Book Workspace Desktop Integration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 将已确认的书籍翻译校对原型接入真实 Tauri 桌面应用，同时保留现有游戏翻译流程。

**Architecture:** 在现有启动页增加书籍工作台入口，游戏入口继续调用原有扫描与翻译流程。书籍工作台作为独立的前端工作区挂载，使用隔离样式与本地样例数据验证混合编辑体验，不修改 Rust 后端命令。

**Tech Stack:** React 19、TypeScript、Vite、Vitest、Testing Library、Tauri 2、原生 CSS。

---

### Task 1: 锁定桌面入口行为

**Files:**
- Create: `apps/desktop/src/test/book-workspace.test.tsx`
- Modify: `apps/desktop/src/App.tsx`
- Modify: `apps/desktop/src/features/projects/ProjectHome.tsx`

1. 编写测试，要求启动页同时保留“选择内容目录”并出现“进入书籍翻译工作台”。
2. 点击书籍入口后，断言出现“书稿、编辑、问题、术语、导出”专属导航。
3. 运行测试，确认因入口不存在而失败。
4. 添加最小视图状态与返回项目中心行为。

### Task 2: 迁移书籍混合编辑工作台

**Files:**
- Create: `apps/desktop/src/features/books/*.tsx`
- Create: `apps/desktop/src/features/books/demoBook.ts`
- Create: `apps/desktop/src/features/books/book-workspace.css`
- Modify: `apps/desktop/src/test/book-workspace.test.tsx`

1. 添加模式切换和 `Ctrl + Enter` 连续校对的失败测试。
2. 迁移阅读编辑、逐段校对、章节树和检查器组件。
3. 将所有样式限定在 `.book-workspace` 内，避免改变游戏工作台。
4. 运行书籍测试和现有项目流程测试。

### Task 3: 生产构建与运行时验收

**Files:**
- Build output: `target/release/game-translator-desktop.exe`

1. 运行 `npm test -- --run` 和 `npm run build`。
2. 运行 `npx --yes @tauri-apps/cli@latest build --no-bundle`。
3. 启动新 EXE，确认启动页有书籍入口，且能进入书籍工作台并返回。
4. 记录文件大小、时间和 SHA256。
