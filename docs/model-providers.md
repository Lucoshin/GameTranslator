# 模型 Provider

## OpenAI-compatible

用户配置 Base URL、API Key 和模型名。实现调用 `/chat/completions`，要求模型返回严格 JSON。该接口可用于兼容 OpenAI Chat Completions 结构的服务，但具体兼容性取决于服务商实现。

## Ollama

调用本地 `/api/chat`，关闭流式响应并要求 JSON 格式。GameTranslator 不负责下载或管理 Ollama 模型。

## 请求内容

请求包含目标语言、稳定 Segment ID 和待翻译文本。正式流程还会加入必要的场景上下文和术语。API Key 不会出现在请求正文、项目数据库或补丁中。

## 错误处理

- HTTP 429 映射为限流错误并进行有限重试。
- 临时传输失败进行有限重试。
- 无效结构不重复盲试，批次会二分拆分。
- 单条仍失败时进入人工处理列表，不生成伪造译文。

