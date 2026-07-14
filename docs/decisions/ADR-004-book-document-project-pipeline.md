# ADR-004: 书籍文档采用只读源文件与独立项目状态

## Status
Accepted

## Date
2026-07-14

## Context
书籍翻译需要导入 TXT、Markdown、EPUB 和 DOCX，保存逐段译文与校对状态，并允许重新打开项目。直接改写源文件会破坏原稿，也难以在不同容器格式之间保持可靠的往返编辑。

## Decision
- 新增 `content-book` 适配器层，将不同文件格式统一解析为 `BookProject → Chapter → Segment`。
- 原始文件始终只读；项目状态以 JSON 保存在应用本地数据目录，源路径仅用于身份与重新导入。
- `SegmentStatus` 的权威值为 `untranslated | draft | reviewed | issue`，Rust DTO 与 TypeScript 使用完全相同的字符串。
- 模型翻译复用现有 `translation-core` 和 Provider 配置，不创建第二套网络客户端。
- 导出始终生成独立文件，不覆盖原稿；出版交付格式与历史策略见 ADR-005。

## Alternatives Considered

### 直接修改 EPUB/DOCX 容器
往返保真成本高，图片、样式、脚注和内部引用容易损坏，因此暂不采用。

### 把书籍伪装成游戏目录
可以复用部分命令，但章节、段落状态和导出语义会泄漏游戏概念，因此拒绝。

### 只把项目状态放在前端 localStorage
无法承载大型书稿，也不适合可靠恢复与后续迁移，因此拒绝。

## Consequences
- 任意支持格式都可以进入同一编辑工作台。
- 用户修改与源文件隔离，导出失败不会损坏原稿。
- 不承诺对导入文件做原格式往返回写；DOCX、EPUB、PDF 与 Markdown 都从统一书稿模型重新生成。
