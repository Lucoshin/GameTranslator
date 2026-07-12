import { useState } from "react";

const OPENAI_COMPATIBLE_PRESETS = [
  ["OpenAI", "https://api.openai.com/v1"],
  ["DeepSeek", "https://api.deepseek.com/v1"],
  ["通义千问（中国大陆）", "https://dashscope.aliyuncs.com/compatible-mode/v1"],
  ["硅基流动", "https://api.siliconflow.cn/v1"],
  ["OpenRouter", "https://openrouter.ai/api/v1"],
] as const;

export type ProviderConfiguration = {
  kind: "openai" | "ollama";
  baseUrl: string;
  model: string;
  apiKey?: string;
  performance?: "stable" | "balanced" | "fast";
};

export function ProviderDrawer({
  open,
  current,
  onClose,
  onSave,
}: {
  open: boolean;
  current: ProviderConfiguration | null;
  onClose: () => void;
  onSave: (configuration: ProviderConfiguration) => void;
}) {
  const [kind, setKind] = useState<"openai" | "ollama">(current?.kind ?? "openai");
  const [baseUrl, setBaseUrl] = useState(current?.baseUrl ?? "https://api.example.com/v1");
  const [apiKey, setApiKey] = useState("");
  const [model, setModel] = useState(current?.model ?? "");
  const [performance, setPerformance] = useState<"stable" | "balanced" | "fast">(current?.performance ?? "balanced");
  if (!open) return null;

  return (
    <div className="drawer-backdrop" onMouseDown={onClose}>
      <section className="provider-drawer" role="dialog" aria-modal="true" aria-label="模型接入" onMouseDown={(event) => event.stopPropagation()}>
        <header><div><span className="eyebrow">MODEL PROVIDER</span><h2>模型接入</h2></div><button onClick={onClose} aria-label="关闭">×</button></header>
        <div className="provider-tabs">
          <button className={kind === "openai" ? "active" : ""} onClick={() => { setKind("openai"); setBaseUrl("https://api.example.com/v1"); }}>OpenAI-compatible</button>
          <button className={kind === "ollama" ? "active" : ""} onClick={() => { setKind("ollama"); setBaseUrl("http://127.0.0.1:11434"); }}>Ollama</button>
        </div>
        <div className="base-url-fields">
          <label>Base URL<input aria-label="Base URL" value={baseUrl} onChange={(event) => setBaseUrl(event.target.value)} /></label>
          {kind === "openai" ? <label>常见格式<select aria-label="常见格式" value={OPENAI_COMPATIBLE_PRESETS.some(([, url]) => url === baseUrl) ? baseUrl : ""} onChange={(event) => { if (event.target.value) setBaseUrl(event.target.value); }}><option value="">自定义</option>{OPENAI_COMPATIBLE_PRESETS.map(([name, url]) => <option key={url} value={url}>{name}</option>)}</select></label> : null}
        </div>
        {kind === "openai" ? <label>API Key<input aria-label="API Key" value={apiKey} onChange={(event) => setApiKey(event.target.value)} type="password" placeholder="sk-••••••••" /></label> : null}
        <label>模型名称<input value={model} onChange={(event) => setModel(event.target.value)} placeholder="例如 deepseek-chat" /></label>
        <label>性能模式<select aria-label="性能模式" value={performance} onChange={(event) => setPerformance(event.target.value as "stable" | "balanced" | "fast")}><option value="stable">稳定 · 低并发</option><option value="balanced">均衡 · 推荐</option><option value="fast">极速 · 高并发</option></select></label>
        <div className="drawer-note"><b>密钥只保存在系统凭据库</b><span>不会写入项目文件或上传到我们的服务器。</span></div>
        <footer><button className="ghost-action" onClick={onClose}>取消</button><button className="primary-action" onClick={() => onSave({ kind, baseUrl, model, performance, apiKey: kind === "openai" && apiKey ? apiKey : undefined })}>保存配置</button></footer>
      </section>
    </div>
  );
}
