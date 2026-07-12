import type { TranslationItem } from "../../App";
import type { Language } from "../translation/LanguageSettings";
import { useDeferredValue, useState } from "react";

const demoRows = [
  ["アリス", "やっと着いた。 \\V[1]", "终于到了。 \\V[1]", "通过"],
  ["アリス", "ここが月の神殿ね。", "这里就是月之神殿。", "术语"],
  ["守衛", "通行証を見せろ。", "出示通行证。", "通过"],
  ["", "中に入る", "进入里面", "通过"],
];

export function SegmentTable({ items, onChange, onExport, targetLanguage, sourceLanguage }: { items?: TranslationItem[]; onChange?: (id: string, target: string) => void; onExport: () => void; targetLanguage: Language; sourceLanguage: Language }) {
  const rows = items?.map((item) => [item.speaker ?? "", item.source, item.target, item.qa === "blocking" ? "阻断" : item.qa === "warning" ? "警告" : "通过", item.sourceFile, item.id] as const) ?? demoRows.map((row, index) => [...row, "Map001", String(index)] as const);
  const warningCount = items?.filter((item) => item.qa !== "passed").length ?? 3;
  const [query, setQuery] = useState("");
  const deferredQuery = useDeferredValue(query.trim().toLocaleLowerCase());
  const [issuesOnly, setIssuesOnly] = useState(false);
  const [page, setPage] = useState(0);
  const filteredRows = rows.filter(([speaker, source, target, qa]) => {
    const matchesQuery = !deferredQuery || `${speaker}\n${source}\n${target}`.toLocaleLowerCase().includes(deferredQuery);
    return matchesQuery && (!issuesOnly || qa !== "通过");
  });
  const pageSize = 100;
  const pageCount = Math.max(1, Math.ceil(filteredRows.length / pageSize));
  const visibleRows = filteredRows.slice(page * pageSize, (page + 1) * pageSize);
  return (
    <div className="page review-page">
      <section className="page-heading compact"><div><p className="kicker">HUMAN REVIEW</p><h1>文本校对</h1><p className="muted">{items ? `${items.length} 条真实翻译结果` : "演示数据"}</p></div><button className="primary-action" onClick={onExport}>导出补丁</button></section>
      <div className="review-toolbar"><div className="search-box">⌕ <input aria-label="搜索文本" value={query} onChange={(event) => { setQuery(event.target.value); setPage(0); }} placeholder="搜索原文、译文或角色…" /></div><button className={issuesOnly ? "filter" : "filter active"} onClick={() => { setIssuesOnly(false); setPage(0); }}>全部 {rows.length}</button><button className={issuesOnly ? "filter active" : "filter"} onClick={() => { setIssuesOnly(true); setPage(0); }}>需检查 {warningCount}</button></div>
      <section className="segment-table" aria-label="翻译文本">
        <header><span>角色 / 位置</span><span>原文</span><span>{targetLanguage.name}</span><span>QA</span></header>
        {visibleRows.map(([speaker, source, target, qa, sourceFile, id], index) => (
          <article key={id} className={qa === "通过" ? "" : "has-warning"}>
            <span><b>{speaker || "选项"}</b><small>{fileName(sourceFile)} · {id.split("::").at(-2) ?? `#${index + 1}`}</small></span>
            <span lang={sourceLanguage.code === "auto" ? undefined : sourceLanguage.code}>{source}</span><span><textarea aria-label={`翻译 ${source}`} value={target} readOnly={!items} onChange={(event) => items && onChange?.(id, event.target.value)} /></span>
            <span><em className={qa === "通过" ? "qa" : "qa warning"}>{qa}</em></span>
          </article>
        ))}
      </section>
      <div className="review-pagination"><span>显示 {filteredRows.length ? page * pageSize + 1 : 0}–{Math.min((page + 1) * pageSize, filteredRows.length)} / {filteredRows.length}</span><button className="filter" disabled={page === 0} onClick={() => setPage((current) => Math.max(0, current - 1))}>上一页</button><button className="filter" disabled={page + 1 >= pageCount} onClick={() => setPage((current) => Math.min(pageCount - 1, current + 1))}>下一页</button></div>
    </div>
  );
}

function fileName(path: string) {
  return path.split(/[\\/]/).at(-1) ?? path;
}
