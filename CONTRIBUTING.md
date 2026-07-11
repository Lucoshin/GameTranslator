# 贡献指南

## 开始之前

1. 阅读产品设计、ADR 和支持矩阵。
2. 为行为变更先创建失败测试。
3. 保持引擎适配、模型 Provider、QA、存储和界面职责分离。
4. 不提交游戏原始资源、API Key、个人路径或未经许可的汉化文件。

## 提交要求

- Rust 代码通过 `cargo fmt --check`、Clippy 和 workspace 测试。
- React 代码通过 Vitest、TypeScript 检查和生产构建。
- 新增引擎字段必须有脱敏 fixture 和往返回写测试。
- 控制码或结构验证失败必须显式报错，不能静默跳过。
- 新功能只实现已确认契约，不添加猜测性兼容字段。

## Commit

使用简洁的 Conventional Commit 风格，例如：

```text
feat: extract rpg maker choices
fix: reject reordered control codes
docs: document patch manifest
```

## Pull Request

PR 描述应包含用户可见变化、测试证据、已知限制和是否改变补丁格式。不要把格式化、重构与行为变更混在同一个 PR 中。

