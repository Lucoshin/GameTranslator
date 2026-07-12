export type ProjectSummary = { projectPath: string; projectName: string; engine: string; segmentCount: number; demo: boolean };

export function ProjectOverview({ project, configured, onConfigure, onStart }: { project: ProjectSummary; configured: boolean; onConfigure: () => void; onStart: () => void }) {
  return (
    <div className="page overview-page">
      {project.demo ? <div className="demo-banner"><span>DEMO</span> 演示数据，不会读取或修改本地文件</div> : null}
      <section className="page-heading">
        <div>
          <p className="kicker">项目已就绪</p>
          <h1>{project.projectName}</h1>
          <p className="muted">{project.projectPath} · <b>{project.engine}</b></p>
        </div>
        <div className="stamp-status">可汉化</div>
      </section>

      <section className="stat-grid">
        <Stat value={project.segmentCount.toLocaleString()} label="可翻译文本" note="已按安全白名单提取" />
        <Stat value={project.engine} label="识别引擎" note="已选择对应引擎适配器" />
        <Stat value="只读" label="源项目策略" note="翻译补丁写入独立目录" />
      </section>

      <div className="overview-grid">
        <section className="panel source-map">
          <div className="panel-title"><span>文本构成</span><small>SCAN RESULT</small></div>
          {project.demo ? <><div className="source-row"><b>地图事件</b><span><i style={{ width: "78%" }} /></span><em>842</em></div><div className="source-row"><b>公共事件</b><span><i style={{ width: "42%" }} /></span><em>196</em></div><div className="source-row"><b>数据库</b><span><i style={{ width: "53%" }} /></span><em>246</em></div></> : <div className="source-row"><b>已提取</b><span><i style={{ width: "100%" }} /></span><em>{project.segmentCount}</em></div>}
        </section>

        <section className="panel launch-panel">
          <div className="panel-title"><span>翻译准备</span><small>READY CHECK</small></div>
          <div className="check-row ok"><span>✓</span><div><b>源文件只读</b><small>将在独立工作目录处理</small></div></div>
          <div className={configured ? "check-row ok" : "check-row"}><span>{configured ? "✓" : "!"}</span><div><b>{configured ? "模型已配置" : "模型尚未配置"}</b><small>支持 OpenAI-compatible / Ollama</small></div></div>
          <button className="secondary-action" onClick={onConfigure}>配置模型</button>
          <button className="primary-action full" aria-label="开始汉化" onClick={onStart}>开始翻译 <span>→</span></button>
        </section>
      </div>
    </div>
  );
}

function Stat({ value, label, note, warning = false }: { value: string; label: string; note: string; warning?: boolean }) {
  return <article className={warning ? "stat-card warning" : "stat-card"}><strong>{value}</strong><span>{label}</span><small>{note}</small></article>;
}
