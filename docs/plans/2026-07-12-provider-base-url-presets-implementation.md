# Provider Base URL Presets Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在模型接入面板中提供常见 API 地址格式的快捷选择，同时始终允许用户编辑 Base URL。

**Architecture:** `ProviderDrawer` 继续以 `baseUrl` 状态作为唯一提交值。新增的原生下拉框仅在用户选择预置项时写入该状态，文本输入框始终绑定并可编辑该状态。

**Tech Stack:** React 19、TypeScript、Vitest、React Testing Library。

---

### Task 1: 覆盖预置选择与自定义编辑

**Files:**
- Create: `apps/desktop/src/test/provider-drawer.test.tsx`
- Modify: `apps/desktop/src/features/providers/ProviderDrawer.tsx`

**Step 1: Write the failing test**

渲染 `ProviderDrawer`，选择“DeepSeek”预置并断言 Base URL 被填为 `https://api.deepseek.com/v1`；随后直接修改 Base URL 并断言新值仍被保留。

**Step 2: Run test to verify it fails**

Run: `npm test -- provider-drawer.test.tsx`

Expected: FAIL，因为预置下拉框尚不存在。

**Step 3: Write minimal implementation**

为 OpenAI-compatible 模式增加“常见格式”原生下拉框；选择项更新现有 `baseUrl` 状态，保留现有可编辑文本输入框。

**Step 4: Run test to verify it passes**

Run: `npm test -- provider-drawer.test.tsx`

Expected: PASS。

**Step 5: Verify integration**

Run: `npm test -- project-flow.test.tsx && npm run typecheck && npm run build`

Expected: 全部退出码为 0。
