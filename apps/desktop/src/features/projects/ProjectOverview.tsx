export function ProjectOverview({ onConfigure, onStart }: { onConfigure: () => void; onStart: () => void }) {
  return (
    <div className="page overview-page">
      <div className="demo-banner"><span>DEMO</span> 演示数据，不会读取或修改本地文件</div>
      <section className="page-heading">
        <div>
          <p className="kicker">项目已就绪</p>
          <h1>月影神殿</h1>
          <p className="muted">D:\Games\Moonlit Shrine · <b>RPG Maker MZ</b></p>
        </div>
        <div className="stamp-status">可汉化</div>
      </section>

      <section className="stat-grid">
        <Stat value="1,284" label="可翻译文本" note="已过滤 316 项技术字段" />
        <Stat value="≈ 38K" label="预计 Token" note="按当前场景批次估算" />
        <Stat value="94%" label="结构覆盖" note="发现 2 个插件动态文本" warning />
      </section>

      <div className="overview-grid">
        <section className="panel source-map">
          <div className="panel-title"><span>文本构成</span><small>SCAN RESULT</small></div>
          <div className="source-row"><b>地图事件</b><span><i style={{ width: "78%" }} /></span><em>842</em></div>
          <div className="source-row"><b>公共事件</b><span><i style={{ width: "42%" }} /></span><em>196</em></div>
          <div className="source-row"><b>数据库</b><span><i style={{ width: "53%" }} /></span><em>246</em></div>
        </section>

        <section className="panel launch-panel">
          <div className="panel-title"><span>翻译准备</span><small>READY CHECK</small></div>
          <div className="check-row ok"><span>✓</span><div><b>源文件只读</b><small>将在独立工作目录处理</small></div></div>
          <div className="check-row"><span>!</span><div><b>模型尚未配置</b><small>支持 OpenAI-compatible / Ollama</small></div></div>
          <button className="secondary-action" onClick={onConfigure}>配置模型</button>
          <button className="primary-action full" aria-label="开始汉化" onClick={onStart}>开始汉化 <span>→</span></button>
        </section>
      </div>
    </div>
  );
}

function Stat({ value, label, note, warning = false }: { value: string; label: string; note: string; warning?: boolean }) {
  return <article className={warning ? "stat-card warning" : "stat-card"}><strong>{value}</strong><span>{label}</span><small>{note}</small></article>;
}
