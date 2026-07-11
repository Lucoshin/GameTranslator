import type { TranslationRun } from "../../App";

export function TranslationProgress({ result, demo, loading, error, onReview, onExport }: { result: TranslationRun | null; demo: boolean; loading: boolean; error: string | null; onReview: () => void; onExport: () => void }) {
  const total = demo ? 1284 : (result?.items.length ?? 0);
  const finished = demo || result !== null;
  return (
    <div className="page task-page">
      <section className="page-heading compact"><div><p className="kicker">TRANSLATION RUN</p><h1>翻译任务</h1><p className="muted">{demo ? "演示任务 · 所有统计均为样例" : "真实项目 · 模型响应将持续写入当前会话"}</p></div><span className="status-pill">{loading ? "翻译中" : finished ? "已完成" : "等待中"}</span></section>
      {error ? <div className="demo-banner" role="alert"><span>ERROR</span>{error}</div> : null}
      <section className="progress-hero">
        <div className="progress-ring"><strong>100<small>%</small></strong></div>
        <div className="progress-copy"><span>{loading ? `0 / ${total}` : `${total} / ${total}`}</span><h2>{loading ? "正在请求模型" : finished ? "场景翻译完成" : "尚未开始"}</h2><p>自动检查发现 {demo ? 3 : (result?.warningFindings ?? 0)} 条普通警告，{demo ? 0 : (result?.blockingFindings ?? 0)} 条阻断错误。</p><div className="progress-actions"><button className="secondary-action" disabled={!finished} onClick={onReview}>校对文本</button><button className="primary-action" disabled={!finished} onClick={onExport}>导出补丁</button></div></div>
      </section>
      <section className="run-metrics"><Metric value={String(total)} label="模型翻译"/><Metric value={demo ? "182" : "0"} label="缓存命中"/><Metric value={String(demo ? 3 : (result?.warningFindings ?? 0))} label="QA 警告" accent/><Metric value={String(demo ? 0 : (result?.failedSegmentIds.length ?? 0))} label="失败"/></section>
      <section className="panel log-panel"><div className="panel-title"><span>任务记录</span><small>LIVE LOG</small></div><Log time="19:42:08" text="完成 Map023.json · 48 个片段"/><Log time="19:42:11" text="校验控制码与结构 · 通过"/><Log time="19:42:12" text="任务完成 · 等待校对或导出" active/></section>
    </div>
  );
}

function Metric({ value, label, accent = false }: { value: string; label: string; accent?: boolean }) { return <div className={accent ? "run-metric accent" : "run-metric"}><strong>{value}</strong><span>{label}</span></div>; }
function Log({ time, text, active = false }: { time: string; text: string; active?: boolean }) { return <div className={active ? "log-row active" : "log-row"}><time>{time}</time><i/><span>{text}</span></div>; }
