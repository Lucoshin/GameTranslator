import type { TranslationItem } from "../../App";
import type { Language } from "../translation/LanguageSettings";

const demoRows = [
  ["アリス", "やっと着いた。 \\V[1]", "终于到了。 \\V[1]", "通过"],
  ["アリス", "ここが月の神殿ね。", "这里就是月之神殿。", "术语"],
  ["守衛", "通行証を見せろ。", "出示通行证。", "通过"],
  ["", "中に入る", "进入里面", "通过"],
];

export function SegmentTable({ items, onChange, onExport, targetLanguage }: { items?: TranslationItem[]; onChange?: (id: string, target: string) => void; onExport: () => void; targetLanguage: Language }) {
  const rows = items?.map((item) => [item.speaker ?? "", item.source, item.target, "通过"] as const) ?? demoRows;
  return (
    <div className="page review-page">
      <section className="page-heading compact"><div><p className="kicker">HUMAN REVIEW</p><h1>文本校对</h1><p className="muted">演示数据 · Map001 / 入口事件</p></div><button className="primary-action" onClick={onExport}>导出补丁</button></section>
      <div className="review-toolbar"><div className="search-box">⌕ <input aria-label="搜索文本" placeholder="搜索原文、译文或角色…" /></div><button className="filter active">全部 1,284</button><button className="filter">警告 3</button><button className="filter">已锁定 26</button></div>
      <section className="segment-table" aria-label="翻译文本">
        <header><span>角色 / 位置</span><span>原文</span><span>{targetLanguage.name}</span><span>QA</span></header>
        {rows.map(([speaker, source, target, qa], index) => (
          <article key={source} className={qa === "术语" ? "has-warning" : ""}>
            <span><b>{speaker || "选项"}</b><small>Map001 · #{index + 12}</small></span>
            <span lang="ja">{source}</span><span><textarea aria-label={`翻译 ${source}`} value={target} readOnly={!items} onChange={(event) => items && onChange?.(items[index].id, event.target.value)} /></span>
            <span><em className={qa === "术语" ? "qa warning" : "qa"}>{qa}</em></span>
          </article>
        ))}
      </section>
    </div>
  );
}
