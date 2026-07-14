# Book Workspace Prototype Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 构建一版可运行、可交互的书籍翻译与校对桌面界面原型，用于验证统一项目中心和混合编辑工作台体验。

**Architecture:** 在 `prototype/` 中创建独立 Vite + React + TypeScript 应用。使用本地样例数据和组件状态模拟项目导航、章节选择、段落编辑、自动保存与校对确认；不连接 Tauri、文件系统或真实模型 API。

**Tech Stack:** React 19、TypeScript、Vite、Vitest、Testing Library、原生 CSS。

---

### Task 1: 初始化原型工程与测试门禁

**Files:**
- Create: `prototype/package.json`
- Create: `prototype/index.html`
- Create: `prototype/tsconfig.json`
- Create: `prototype/vite.config.ts`
- Create: `prototype/src/main.tsx`
- Create: `prototype/src/App.test.tsx`
- Create: `prototype/src/test/setup.ts`

**Step 1:** 创建依赖、Vite、TypeScript 和 Vitest 配置。

**Step 2:** 编写失败的应用烟雾测试，要求出现“项目”导航与“书籍”项目类型。

**Step 3:** 运行 `npm test -- --run`，确认测试因 `App` 尚不存在而失败。

**Step 4:** 创建最小 `App.tsx` 使烟雾测试通过。

**Step 5:** 再次运行测试并提交 `chore: scaffold book workspace prototype`。

### Task 2: 实现项目中心与书籍工作台壳层

**Files:**
- Create: `prototype/src/data/demoBook.ts`
- Create: `prototype/src/components/Icon.tsx`
- Create: `prototype/src/components/ProjectCenter.tsx`
- Create: `prototype/src/components/BookWorkspace.tsx`
- Modify: `prototype/src/App.tsx`
- Modify: `prototype/src/App.test.tsx`

**Step 1:** 编写失败测试：点击书籍项目后显示书稿、编辑、问题、术语、导出导航。

**Step 2:** 运行测试并确认因工作台未实现而失败。

**Step 3:** 添加样例书稿数据、统一 SVG 图标组件、项目中心和工作台壳层。

**Step 4:** 运行测试确认通过并提交 `feat: add project center and book workspace shell`。

### Task 3: 实现混合编辑交互

**Files:**
- Create: `prototype/src/components/ChapterTree.tsx`
- Create: `prototype/src/components/ReadingEditor.tsx`
- Create: `prototype/src/components/SegmentReview.tsx`
- Create: `prototype/src/components/Inspector.tsx`
- Modify: `prototype/src/components/BookWorkspace.tsx`
- Modify: `prototype/src/App.test.tsx`

**Step 1:** 编写失败测试：选择段落后切换到逐段校对，当前段落保持选中。

**Step 2:** 编写失败测试：修改译文显示保存反馈，`Ctrl + Enter` 将当前段落标记为已校对并前往下一段。

**Step 3:** 运行测试并确认均因行为缺失而失败。

**Step 4:** 实现章节树、阅读编辑、逐段校对、检查器和共享选择状态。

**Step 5:** 运行测试确认通过并提交 `feat: add hybrid book editing interactions`。

### Task 4: 完成品牌视觉与响应式体验

**Files:**
- Create: `prototype/src/styles/tokens.css`
- Create: `prototype/src/styles/global.css`
- Modify: `prototype/src/main.tsx`
- Modify: `prototype/src/components/BookWorkspace.tsx`

**Step 1:** 添加暖灰、纸白、墨色、朱红语义变量和出版编辑风格排版。

**Step 2:** 实现三栏布局、段落状态轨道、可见焦点、150–250ms 过渡和减少动态效果支持。

**Step 3:** 添加 1400px、1100px 与窄窗口响应式规则，确保编辑区始终保持可读。

**Step 4:** 运行 `npm test -- --run` 与 `npm run build`，提交 `feat: style responsive editorial workspace`。

### Task 5: 浏览器视觉检查与修正

**Files:**
- Modify: `prototype/src/styles/global.css`（如需要）
- Modify: `prototype/src/components/*.tsx`（如需要）

**Step 1:** 启动本地预览并在 1440×900、1280×720 和窄窗口下截图。

**Step 2:** 检查文本对比度、溢出、焦点、侧栏折叠、阅读行宽和模式切换。

**Step 3:** 对发现的问题先补失败测试，再进行最小修正。

**Step 4:** 运行完整测试和构建，提交 `fix: polish book workspace prototype`（仅在有修正时）。
