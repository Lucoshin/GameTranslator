import type { TranslationItem } from "../../App";

export type ProjectSummary = { projectPath: string; projectName: string; engine: string; segmentCount: number; previewItems?: TranslationItem[] };

export function ProjectOverview({ project, configured, scanning, onSelect, onStart }: { project: ProjectSummary | null; configured: boolean; scanning: boolean; onSelect: () => void; onStart: () => void }) {
  if (!project) {
    return <div className="page overview-page empty-project-overview">
      <section className="page-heading">
        <div><p className="kicker">PROJECT WORKSPACE</p><h1>开始一个项目</h1><p className="muted">先选择受支持的游戏或模组目录，扫描完成后即可在此配置语言、预览原文并开始翻译。</p></div>
        <div className="stamp-status">待选择</div>
      </section>
      <section className="panel empty-project-panel"><div><b>尚未选择内容目录</b><p>支持 RPG Maker MV / MZ、Ren'Py 与 RimWorld 模组；原始内容始终保持只读，补丁会导出到独立目录。</p></div><button className="primary-action" aria-label="概览页选择内容目录" disabled={scanning} onClick={onSelect}>{scanning ? "正在识别…" : "选择游戏或模组目录"} <span>→</span></button></section>
    </div>;
  }
  return (
    <div className="page overview-page">
      <section className="page-heading">
        <div>
          <p className="kicker">项目已就绪</p>
          <h1>{project.projectName}</h1>
          <p className="muted">{project.projectPath} · <b>{project.engine}</b></p>
        </div>
        <div className="overview-heading-actions"><button className="secondary-action" aria-label="概览页选择内容目录" disabled={scanning} onClick={onSelect}>{scanning ? "正在识别…" : "选择内容目录"}</button><div className="stamp-status">可翻译</div></div>
      </section>

      <section className="stat-grid">
        <Stat value={project.segmentCount.toLocaleString()} label="可翻译文本" note="已按安全白名单提取" />
        <Stat value={project.engine} label="识别引擎" note="已选择对应引擎适配器" />
        <Stat value="只读" label="源项目策略" note="翻译补丁写入独立目录" />
      </section>

      <div className="overview-grid">
        <section className="panel source-map">
          <div className="panel-title"><span>文本构成</span><small>SCAN RESULT</small></div>
          <div className="source-row"><b>已提取</b><span><i style={{ width: "100%" }} /></span><em>{project.segmentCount}</em></div>
        </section>

        <section className="panel launch-panel">
          <div className="panel-title"><span>翻译准备</span><small>READY CHECK</small></div>
          <div className="check-row ok"><span>✓</span><div><b>源文件只读</b><small>将在独立工作目录处理</small></div></div>
          <div className={configured ? "check-row ok" : "check-row"}><span>{configured ? "✓" : "!"}</span><div><b>{configured ? "模型已配置" : "模型尚未配置"}</b><small>支持 OpenAI-compatible / Ollama</small></div></div>
          <button className="primary-action full" aria-label="开始翻译" onClick={onStart}>开始翻译 <span>→</span></button>
        </section>
      </div>
    </div>
  );
}

function Stat({ value, label, note, warning = false }: { value: string; label: string; note: string; warning?: boolean }) {
  return <article className={warning ? "stat-card warning" : "stat-card"}><strong>{value}</strong><span>{label}</span><small>{note}</small></article>;
}
