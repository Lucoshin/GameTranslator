import { useState } from "react";

export type ProviderConfiguration = {
  kind: "openai" | "ollama";
  baseUrl: string;
  model: string;
  apiKey?: string;
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
  if (!open) return null;

  return (
    <div className="drawer-backdrop" onMouseDown={onClose}>
      <section className="provider-drawer" role="dialog" aria-modal="true" aria-label="模型接入" onMouseDown={(event) => event.stopPropagation()}>
        <header><div><span className="eyebrow">MODEL PROVIDER</span><h2>模型接入</h2></div><button onClick={onClose} aria-label="关闭">×</button></header>
        <div className="provider-tabs">
          <button className={kind === "openai" ? "active" : ""} onClick={() => { setKind("openai"); setBaseUrl("https://api.example.com/v1"); }}>OpenAI-compatible</button>
          <button className={kind === "ollama" ? "active" : ""} onClick={() => { setKind("ollama"); setBaseUrl("http://127.0.0.1:11434"); }}>Ollama</button>
        </div>
        <label>Base URL<input aria-label="Base URL" value={baseUrl} onChange={(event) => setBaseUrl(event.target.value)} /></label>
        {kind === "openai" ? <label>API Key<input aria-label="API Key" value={apiKey} onChange={(event) => setApiKey(event.target.value)} type="password" placeholder="sk-••••••••" /></label> : null}
        <label>模型名称<input value={model} onChange={(event) => setModel(event.target.value)} placeholder="例如 deepseek-chat" /></label>
        <div className="drawer-note"><b>密钥只保存在系统凭据库</b><span>不会写入项目文件或上传到我们的服务器。</span></div>
        <footer><button className="ghost-action" onClick={onClose}>取消</button><button className="primary-action" onClick={() => onSave({ kind, baseUrl, model, apiKey: kind === "openai" && apiKey ? apiKey : undefined })}>保存配置</button></footer>
      </section>
    </div>
  );
}
