import type { TranslationLog, TranslationProgressState, TranslationRun } from "../../App";
import type { CSSProperties } from "react";
import { useEffect, useState } from "react";

export function TranslationProgress({ result, loading, error, progress, logs, sourceCount, onStart, onReview, onExport }: { result: TranslationRun | null; loading: boolean; error: string | null; progress: TranslationProgressState | null; logs: TranslationLog[]; sourceCount: number; onStart: () => void; onReview: () => void; onExport: () => void }) {
  const [elapsedSeconds, setElapsedSeconds] = useState(0);
  useEffect(() => {
    if (!loading) return;
    const startedAt = Date.now();
    setElapsedSeconds(0);
    const timer = window.setInterval(() => setElapsedSeconds(Math.floor((Date.now() - startedAt) / 1000)), 1000);
    return () => window.clearInterval(timer);
  }, [loading]);
  const waiting = !loading && result === null;
  const total = progress?.total ?? (result?.items.length ?? sourceCount);
  const completed = progress?.completed ?? (result?.items.length ?? 0);
  const percent = total > 0 ? Math.min(100, Math.round((completed / total) * 100)) : null;
  const finished = result !== null;
  const warnings = progress?.warningFindings ?? (result?.warningFindings ?? 0);
  const blocking = progress?.blockingFindings ?? (result?.blockingFindings ?? 0);
  const failed = progress?.failed ?? (result?.failedSegmentIds.length ?? 0);
  const elapsed = `${String(Math.floor(elapsedSeconds / 60)).padStart(2, "0")}:${String(elapsedSeconds % 60).padStart(2, "0")}`;
  const eta = formatDuration(progress?.etaSeconds ?? 0);
  const phaseTitle = waiting ? "准备开始翻译" : progress?.phase === "extracting" ? "正在提取游戏文本" : progress?.phase === "qa" ? "正在执行质量检查" : progress?.phase === "failed" ? "任务失败" : finished ? "场景翻译完成" : "正在请求模型";
  const status = waiting ? "待开始" : progress?.phase === "failed" ? "失败" : loading ? progress?.phase === "qa" ? "检查中" : progress?.phase === "extracting" ? "提取中" : "翻译中" : finished ? "已完成" : "等待中";
  return (
    <div className="page task-page">
      <section className="page-heading compact"><div><p className="kicker">TRANSLATION RUN</p><h1>翻译任务</h1><p className="muted">批次进度和质量检查由后端实时上报</p></div><span className="status-pill">{status}</span></section>
      {error ? <div className="notice-banner" role="alert"><span>ERROR</span>{error}</div> : null}
      <section className="progress-hero">
        <div className={`${waiting ? "progress-ring ready" : percent === null ? "progress-ring indeterminate" : "progress-ring"}${loading ? " working" : ""}`} style={{ "--progress": `${percent ?? 0}%` } as CSSProperties}><strong>{waiting ? "待" : percent === null ? "…" : `${percent}%`}</strong></div>
        <div className="progress-copy"><span>{waiting ? `${sourceCount} 条原文已提取` : total > 0 ? `${completed} / ${total}` : "正在统计文本"}{loading ? ` · 已运行 ${elapsed}` : ""}</span><h2>{phaseTitle}</h2><p>{waiting ? "确认模型配置后即可开始翻译；你也可以先查看已提取的原文。" : progress?.message ?? `自动检查发现 ${warnings} 条普通警告，${blocking} 条阻断错误。`}</p>{loading && progress?.throughput ? <div className="live-speed"><b>{progress.concurrency ?? 0} 路</b><b>{progress.throughput.toFixed(1)} 片段/秒</b><b>预计 {eta}</b></div> : null}<div className="progress-actions">{waiting ? <><button className="secondary-action" onClick={onReview}>查看原文</button><button className="primary-action" onClick={onStart}>开始翻译 <span>→</span></button></> : <><button className="secondary-action" disabled={!finished} onClick={onReview}>校对文本</button><button className="primary-action" disabled={!finished} onClick={onExport}>导出补丁</button></>}</div></div>
      </section>
      <section className="run-metrics"><Metric value={String(completed)} label="已处理"/><Metric value={String(Math.max(0, total - completed))} label="待处理"/><Metric value={String(warnings)} label="QA 警告" accent/><Metric value={String(failed)} label="失败"/></section>
      <section className="panel log-panel"><div className="panel-title"><span>任务记录</span><small>LIVE LOG</small></div><div className="log-scroll">{logs.length ? logs.map((log, index) => <Log key={`${log.time}-${index}`} time={log.time} text={log.message} active={index === logs.length - 1}/>) : <p className="muted">尚无任务事件</p>}</div></section>
    </div>
  );
}

function Metric({ value, label, accent = false }: { value: string; label: string; accent?: boolean }) { return <div className={accent ? "run-metric accent" : "run-metric"}><strong>{value}</strong><span>{label}</span></div>; }
function Log({ time, text, active = false }: { time: string; text: string; active?: boolean }) { return <div className={active ? "log-row active" : "log-row"}><time>{time}</time><i/><span>{text}</span></div>; }
function formatDuration(seconds: number) { return `${String(Math.floor(seconds / 60)).padStart(2, "0")}:${String(Math.round(seconds % 60)).padStart(2, "0")}`; }
