import type { TranslationLog, TranslationProgressState, TranslationRun } from "../../App";
import type { CSSProperties } from "react";

export function TranslationProgress({ result, demo, loading, error, progress, logs, onReview, onExport }: { result: TranslationRun | null; demo: boolean; loading: boolean; error: string | null; progress: TranslationProgressState | null; logs: TranslationLog[]; onReview: () => void; onExport: () => void }) {
  const total = progress?.total ?? (demo ? 1284 : (result?.items.length ?? 0));
  const completed = progress?.completed ?? (result?.items.length ?? 0);
  const percent = total > 0 ? Math.min(100, Math.round((completed / total) * 100)) : null;
  const finished = demo || result !== null;
  const warnings = progress?.warningFindings ?? (demo ? 3 : (result?.warningFindings ?? 0));
  const blocking = progress?.blockingFindings ?? (demo ? 0 : (result?.blockingFindings ?? 0));
  const failed = progress?.failed ?? (result?.failedSegmentIds.length ?? 0);
  const phaseTitle = progress?.phase === "extracting" ? "正在提取游戏文本" : progress?.phase === "qa" ? "正在执行质量检查" : progress?.phase === "failed" ? "任务失败" : finished ? "场景翻译完成" : "正在请求模型";
  const status = progress?.phase === "failed" ? "失败" : loading ? progress?.phase === "qa" ? "检查中" : progress?.phase === "extracting" ? "提取中" : "翻译中" : finished ? "已完成" : "等待中";
  return (
    <div className="page task-page">
      <section className="page-heading compact"><div><p className="kicker">TRANSLATION RUN</p><h1>翻译任务</h1><p className="muted">{demo ? "演示任务 · 所有统计均为样例" : "真实项目 · 批次进度和质量检查由后端实时上报"}</p></div><span className="status-pill">{status}</span></section>
      {error ? <div className="demo-banner" role="alert"><span>ERROR</span>{error}</div> : null}
      <section className="progress-hero">
        <div className={percent === null ? "progress-ring indeterminate" : "progress-ring"} style={{ "--progress": `${percent ?? 0}%` } as CSSProperties}><strong>{percent === null ? "…" : `${percent}%`}</strong></div>
        <div className="progress-copy"><span>{total > 0 ? `${completed} / ${total}` : "正在统计文本"}</span><h2>{phaseTitle}</h2><p>自动检查发现 {warnings} 条普通警告，{blocking} 条阻断错误。</p><div className="progress-actions"><button className="secondary-action" disabled={!finished} onClick={onReview}>校对文本</button><button className="primary-action" disabled={!finished} onClick={onExport}>导出补丁</button></div></div>
      </section>
      <section className="run-metrics"><Metric value={String(completed)} label="已处理"/><Metric value={String(Math.max(0, total - completed))} label="待处理"/><Metric value={String(warnings)} label="QA 警告" accent/><Metric value={String(failed)} label="失败"/></section>
      <section className="panel log-panel"><div className="panel-title"><span>任务记录</span><small>LIVE LOG</small></div>{logs.length ? logs.map((log, index) => <Log key={`${log.time}-${index}`} time={log.time} text={log.message} active={index === logs.length - 1}/>) : <p className="muted">尚无任务事件</p>}</section>
    </div>
  );
}

function Metric({ value, label, accent = false }: { value: string; label: string; accent?: boolean }) { return <div className={accent ? "run-metric accent" : "run-metric"}><strong>{value}</strong><span>{label}</span></div>; }
function Log({ time, text, active = false }: { time: string; text: string; active?: boolean }) { return <div className={active ? "log-row active" : "log-row"}><time>{time}</time><i/><span>{text}</span></div>; }
