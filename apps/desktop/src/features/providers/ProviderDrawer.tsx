import { useState } from "react";

export function ProviderDrawer({
  open,
  currentModel,
  onClose,
  onSave,
}: {
  open: boolean;
  currentModel: string;
  onClose: () => void;
  onSave: (model: string) => void;
}) {
  const [model, setModel] = useState(currentModel);
  if (!open) return null;

  return (
    <div className="drawer-backdrop" onMouseDown={onClose}>
      <section className="provider-drawer" role="dialog" aria-modal="true" aria-label="模型接入" onMouseDown={(event) => event.stopPropagation()}>
        <header><div><span className="eyebrow">MODEL PROVIDER</span><h2>模型接入</h2></div><button onClick={onClose} aria-label="关闭">×</button></header>
        <div className="provider-tabs"><button className="active">OpenAI-compatible</button><button>Ollama</button></div>
        <label>Base URL<input defaultValue="https://api.example.com/v1" /></label>
        <label>API Key<input type="password" placeholder="sk-••••••••" /></label>
        <label>模型名称<input value={model} onChange={(event) => setModel(event.target.value)} placeholder="例如 deepseek-chat" /></label>
        <div className="drawer-note"><b>密钥只保存在系统凭据库</b><span>不会写入项目文件或上传到我们的服务器。</span></div>
        <footer><button className="ghost-action" onClick={onClose}>取消</button><button className="primary-action" onClick={() => onSave(model || "未配置")}>保存配置</button></footer>
      </section>
    </div>
  );
}

