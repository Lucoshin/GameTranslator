import type { TranslationItem } from "../../App";
import type { Language } from "../translation/LanguageSettings";
import { useDeferredValue, useState } from "react";

type TranslationChange = { id: string; target: string };

export function SegmentTable({ items, translated, onApply, onExport, targetLanguage, sourceLanguage }: { items: TranslationItem[]; translated: boolean; onApply: (changes: TranslationChange[]) => void; onExport: () => void; targetLanguage: Language; sourceLanguage: Language }) {
  const [drafts, setDrafts] = useState<Record<string, string>>({});
  const changes = items.flatMap((item) => {
    const target = drafts[item.id];
    return target !== undefined && target !== item.target ? [{ id: item.id, target }] : [];
  });
  const rows = items.map((item) => [item.speaker ?? "", item.source, drafts[item.id] ?? item.target, translated ? item.qa === "blocking" ? "阻断" : item.qa === "warning" ? "警告" : "通过" : "待翻译", item.sourceFile, item.id] as const);
  const warningCount = translated ? items.filter((item) => item.qa !== "passed").length : 0;
  const [query, setQuery] = useState("");
  const deferredQuery = useDeferredValue(query.trim().toLocaleLowerCase());
  const [issuesOnly, setIssuesOnly] = useState(false);
  const [locationFilter, setLocationFilter] = useState("");
  const [page, setPage] = useState(0);
  const speakers = [...new Set(rows.map(([speaker]) => speaker).filter(Boolean))];
  const sourceFiles = [...new Set(rows.map(([, , , , sourceFile]) => sourceFile))];
  const filteredRows = rows.filter(([speaker, source, target, qa, sourceFile]) => {
    const matchesQuery = !deferredQuery || `${speaker}\n${source}\n${target}`.toLocaleLowerCase().includes(deferredQuery);
    const matchesLocation = !locationFilter
      || (locationFilter.startsWith("speaker:") ? speaker === locationFilter.slice(8) : sourceFile === locationFilter.slice(5));
    return matchesQuery && matchesLocation && (!issuesOnly || qa !== "通过");
  });
  const pageSize = 100;
  const pageCount = Math.max(1, Math.ceil(filteredRows.length / pageSize));
  const visibleRows = filteredRows.slice(page * pageSize, (page + 1) * pageSize);
  const updateDraft = (id: string, target: string) => {
    const item = items.find((candidate) => candidate.id === id);
    if (!item) return;
    setDrafts((current) => {
      if (target !== item.target) return { ...current, [id]: target };
      const { [id]: _, ...remaining } = current;
      return remaining;
    });
  };
  const applyChanges = () => {
    if (!changes.length) return;
    onApply(changes);
    setDrafts({});
  };
  return (
    <div className="page review-page">
      <section className="page-heading compact"><div><p className="kicker">HUMAN REVIEW</p><h1>文本校对</h1><p className="muted">{translated ? `${items.length} 条翻译结果${changes.length ? ` · ${changes.length} 项修改待应用` : ""}` : `${items.length} 条已提取原文 · 尚未调用模型`}</p></div><div className="review-actions"><button className="secondary-action" aria-label={`应用校对修改（${changes.length}）`} disabled={!translated || !changes.length} onClick={applyChanges}>应用修改（{changes.length}）</button><button className="primary-action" disabled={!translated || changes.length > 0} onClick={onExport}>导出补丁</button></div></section>
      <div className="review-toolbar"><div className="search-box">⌕ <input aria-label="搜索文本" value={query} onChange={(event) => { setQuery(event.target.value); setPage(0); }} placeholder="搜索原文、译文或角色…" /></div><button className={issuesOnly ? "filter" : "filter active"} onClick={() => { setIssuesOnly(false); setPage(0); }}>全部 {rows.length}</button><button className={issuesOnly ? "filter active" : "filter"} onClick={() => { setIssuesOnly(true); setPage(0); }}>需检查 {warningCount}</button></div>
      <section className="segment-table" aria-label="翻译文本">
        <header><span className="location-filter"><select aria-label="按角色或位置筛选" value={locationFilter} onChange={(event) => { setLocationFilter(event.target.value); setPage(0); }}><option value="">角色 / 位置 · 全部</option><optgroup label="角色">{speakers.map((speaker) => <option key={speaker} value={`speaker:${speaker}`}>{speaker}</option>)}</optgroup><optgroup label="位置">{sourceFiles.map((sourceFile) => <option key={sourceFile} value={`file:${sourceFile}`}>{fileName(sourceFile)}</option>)}</optgroup></select></span><span>原文</span><span>{targetLanguage.name}</span><span>QA</span></header>
        {visibleRows.map(([speaker, source, target, qa, sourceFile, id], index) => (
          <article key={id} className={qa === "通过" ? "" : "has-warning"}>
            <span><b>{speaker || "选项"}</b><small>{fileName(sourceFile)} · {id.split("::").at(-2) ?? `#${index + 1}`}</small></span>
            <span lang={sourceLanguage.code === "auto" ? undefined : sourceLanguage.code}>{source}</span><span><textarea className={drafts[id] !== undefined ? "edited" : ""} aria-label={`翻译 ${source}`} value={target} placeholder={translated ? undefined : "等待模型翻译"} disabled={!translated} onChange={(event) => updateDraft(id, event.target.value)} /></span>
            <span><em className={qa === "通过" ? "qa" : "qa warning"}>{qa}</em></span>
          </article>
        ))}
        {!visibleRows.length ? <p className="empty-review">暂无可校对的翻译结果</p> : null}
      </section>
      <div className="review-pagination"><span>显示 {filteredRows.length ? page * pageSize + 1 : 0}–{Math.min((page + 1) * pageSize, filteredRows.length)} / {filteredRows.length}</span><button className="filter" disabled={page === 0} onClick={() => setPage((current) => Math.max(0, current - 1))}>上一页</button><button className="filter" disabled={page + 1 >= pageCount} onClick={() => setPage((current) => Math.min(pageCount - 1, current + 1))}>下一页</button></div>
    </div>
  );
}

function fileName(path: string) {
  return path.split(/[\\/]/).at(-1) ?? path;
}
