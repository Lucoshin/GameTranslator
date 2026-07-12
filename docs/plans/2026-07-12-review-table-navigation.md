# 校对表格与概览往返 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 让校对列表在自身区域滚动、支持按角色或位置筛选，并让左上品牌按钮在全局首页与项目概览之间往返。

**Architecture:** 筛选状态保留在 `SegmentTable` 内，根据现有 `speaker` 与 `sourceFile` 派生唯一选项；滚动只由表格容器承担。`App` 在打开全局首页时保留当前项目，首页品牌按钮可回到该项目概览；无项目时首页继续提供目录选择入口。

**Tech Stack:** React 19、TypeScript、CSS、Vitest、Testing Library

---

### Task 1: 校对筛选与表内滚动

**Files:**
- Modify: `apps/desktop/src/features/review/SegmentTable.tsx`
- Modify: `apps/desktop/src/styles/global.css`
- Test: `apps/desktop/src/test/project-flow.test.tsx`

1. 先写测试，断言角色/位置下拉可筛掉不匹配行。
2. 运行该测试并确认因缺少下拉失败。
3. 添加派生筛选项、筛选状态与固定高度滚动容器。
4. 运行测试并确认通过。

### Task 2: 品牌按钮往返全局首页与项目概览

**Files:**
- Modify: `apps/desktop/src/App.tsx`
- Test: `apps/desktop/src/test/project-flow.test.tsx`

1. 先写测试，断言从项目概览点击品牌按钮进入全局首页，再次点击返回原项目概览。
2. 运行该测试并确认缺少全局首页切换入口。
3. 首页显示时保留当前项目，并为首页品牌按钮提供返回项目概览入口。
4. 运行前端测试、类型检查与构建。
