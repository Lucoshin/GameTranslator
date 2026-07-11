export function TranslationProgress({ onReview, onExport }: { onReview: () => void; onExport: () => void }) {
  return (
    <div className="page task-page">
      <section className="page-heading compact"><div><p className="kicker">TRANSLATION RUN</p><h1>翻译任务</h1><p className="muted">演示任务 · 所有统计均为样例</p></div><span className="status-pill">已完成</span></section>
      <section className="progress-hero">
        <div className="progress-ring"><strong>100<small>%</small></strong></div>
        <div className="progress-copy"><span>1,284 / 1,284</span><h2>场景翻译完成</h2><p>自动检查发现 3 条普通警告，没有阻断错误。</p><div className="progress-actions"><button className="secondary-action" onClick={onReview}>校对文本</button><button className="primary-action" onClick={onExport}>导出补丁</button></div></div>
      </section>
      <section className="run-metrics"><Metric value="1,102" label="模型翻译"/><Metric value="182" label="缓存命中"/><Metric value="3" label="QA 警告" accent/><Metric value="0" label="失败"/></section>
      <section className="panel log-panel"><div className="panel-title"><span>任务记录</span><small>LIVE LOG</small></div><Log time="19:42:08" text="完成 Map023.json · 48 个片段"/><Log time="19:42:11" text="校验控制码与结构 · 通过"/><Log time="19:42:12" text="任务完成 · 等待校对或导出" active/></section>
    </div>
  );
}

function Metric({ value, label, accent = false }: { value: string; label: string; accent?: boolean }) { return <div className={accent ? "run-metric accent" : "run-metric"}><strong>{value}</strong><span>{label}</span></div>; }
function Log({ time, text, active = false }: { time: string; text: string; active?: boolean }) { return <div className={active ? "log-row active" : "log-row"}><time>{time}</time><i/><span>{text}</span></div>; }

