import { useEffect, useRef, useState, type KeyboardEvent } from "react";
import { bookProgress, type BookChapter, type BookExportFormat, type BookExportProfile, type BookExportRecord, type BookProject, type BookSegment, type PrintPreset, type PublicationMetadata } from "./contracts";
import type { WorkspaceStage } from "../workspace/WorkspaceChrome";
import { sourceLanguages, targetLanguages, type Language } from "../translation/LanguageSettings";
import "./book-workspace.css";

export function BookWorkspace({ project, stage, providerName, busy, error, exportHistory, onSave, onTranslate, onExport, onOpenExport }: {
  project: BookProject;
  stage: WorkspaceStage;
  providerName: string | null;
  busy: boolean;
  error: string | null;
  exportHistory: BookExportRecord[];
  onSave: (project: BookProject) => Promise<void>;
  onTranslate: (project: BookProject, chapterId: string | null) => void;
  onExport: (project: BookProject, format: BookExportFormat, profile: BookExportProfile) => void;
  onOpenExport: (path: string) => void;
}) {
  const [working, setWorking] = useState(project);
  const [chapterId, setChapterId] = useState(project.chapters[0]?.id ?? "");
  const [mode, setMode] = useState<"reading" | "review">("reading");
  const [selectedId, setSelectedId] = useState(project.chapters[0]?.segments[0]?.id ?? "");
  const [saveState, setSaveState] = useState<"saved" | "saving" | "error">("saved");
  const saveTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  useEffect(() => {
    setWorking(project);
    setChapterId((current) => project.chapters.some((chapter) => chapter.id === current) ? current : (project.chapters[0]?.id ?? ""));
  }, [project]);
  useEffect(() => {
    if (stage === "review") setMode("review");
    if (stage === "translation") setMode("reading");
  }, [stage]);
  useEffect(() => () => clearTimeout(saveTimer.current), []);

  const chapter = working.chapters.find((candidate) => candidate.id === chapterId) ?? working.chapters[0];
  const selected = chapter?.segments.find((segment) => segment.id === selectedId) ?? chapter?.segments[0];
  const selectedIndex = Math.max(0, chapter?.segments.findIndex((segment) => segment.id === selected?.id) ?? 0);
  const chapterIndex = Math.max(0, working.chapters.findIndex((candidate) => candidate.id === chapter?.id));

  const persistSoon = (next: BookProject) => {
    setSaveState("saving");
    clearTimeout(saveTimer.current);
    saveTimer.current = setTimeout(() => {
      void onSave(next).then(() => setSaveState("saved")).catch(() => setSaveState("error"));
    }, 450);
  };

  const updateSegment = (id: string, update: (segment: BookSegment) => BookSegment) => {
    setSelectedId(id);
    setWorking((current) => {
      const next = {
        ...current,
        chapters: current.chapters.map((item) => item.id === chapter?.id
          ? { ...item, segments: item.segments.map((segment) => segment.id === id ? update(segment) : segment) }
          : item),
      };
      persistSoon(next);
      return next;
    });
  };

  const updateTranslation = (id: string, value: string) => updateSegment(id, (segment) => ({
    ...segment,
    translation: value,
    status: segment.status === "reviewed" ? "draft" : segment.status,
  }));
  const confirmSegment = (id: string) => {
    updateSegment(id, (segment) => ({ ...segment, status: "reviewed", qaNote: null }));
    const currentIndex = chapter?.segments.findIndex((segment) => segment.id === id) ?? -1;
    const next = chapter?.segments[currentIndex + 1];
    if (next) setSelectedId(next.id);
  };
  const selectChapter = (next: BookChapter) => {
    setChapterId(next.id);
    setSelectedId(next.segments[0]?.id ?? "");
  };
  const updateLanguage = (field: "sourceLanguage" | "targetLanguage", code: string) => {
    setWorking((current) => {
      const next = { ...current, [field]: code };
      persistSoon(next);
      return next;
    });
  };
  const updatePublication = (field: keyof PublicationMetadata, value: string) => {
    setWorking((current) => {
      const next = { ...current, publication: { ...current.publication, [field]: value } };
      persistSoon(next);
      return next;
    });
  };

  const allSegments = working.chapters.flatMap((item) => item.segments);
  const reviewedCount = allSegments.filter((segment) => segment.status === "reviewed").length;
  const issueCount = allSegments.filter((segment) => segment.status === "issue").length;
  const translatedCount = allSegments.filter((segment) => segment.translation.trim().length > 0).length;

  if (stage === "overview") {
    return <BookOverview project={working} translatedCount={translatedCount} reviewedCount={reviewedCount} issueCount={issueCount} busy={busy} error={error} onLanguageChange={updateLanguage} onTranslate={onTranslate} />;
  }
  if (stage === "export") {
    return <BookExport project={working} busy={busy} issueCount={issueCount} onPublicationChange={updatePublication} onExport={onExport} />;
  }
  if (stage === "history") {
    return <BookHistory project={working} records={exportHistory} busy={busy} onExport={onExport} onOpenExport={onOpenExport} />;
  }
  if (!chapter || !selected) {
    return <main className="book-workspace embedded"><div className="book-empty-state"><strong>书稿没有可编辑段落</strong></div></main>;
  }
  const chapterIssues = chapter.segments.filter((segment) => segment.status === "issue").length;
  const saveText = saveState === "saving" ? "正在保存…" : saveState === "error" ? "保存失败" : "已保存";

  return (
    <main className="book-workspace embedded">
      <div className="book-toolbar">
        <div><strong>第 {chapterIndex + 1} 章 · {chapter.title}</strong><span>{chapter.segments.length} 个段落 · {chapterIssues} 个待处理问题</span><span className={`book-save-state ${saveState}`}>{saveText}</span>{error ? <span className="book-operation-error" role="alert">{error}</span> : null}</div>
        <div className="book-toolbar-actions">
          <BookLanguageControls project={working} onChange={updateLanguage} />
          <div className="book-mode-switch" aria-label="编辑模式">
            <button type="button" aria-pressed={mode === "reading"} onClick={() => setMode("reading")}>阅读编辑</button>
            <button type="button" aria-pressed={mode === "review"} onClick={() => setMode("review")}>逐段校对</button>
          </div>
          <button type="button" className="book-primary-action" disabled={busy} onClick={() => onTranslate(working, chapter.id)}>{busy ? "翻译中…" : "翻译本章"}</button>
        </div>
      </div>
      <div className="book-grid">
        <ChapterTree project={working} chapterId={chapter.id} onSelect={selectChapter} />
        <div className="book-editor-stage">
          {mode === "reading"
            ? <ReadingEditor chapter={chapter} chapterIndex={chapterIndex} selectedId={selected.id} onSelect={setSelectedId} onChange={updateTranslation} onConfirm={confirmSegment} />
            : <SegmentReview segments={chapter.segments} selectedId={selected.id} onSelect={setSelectedId} onChange={updateTranslation} onConfirm={confirmSegment} />}
        </div>
        <Inspector segment={selected} index={selectedIndex} providerName={providerName} />
      </div>
      <footer className="book-status-bar"><span><i /> {working.sourcePath} · 自动保存</span><span>第 {chapterIndex + 1} / {working.chapters.length} 章</span><span>本章 {chapter.segments.reduce((count, segment) => count + segment.translation.length, 0)} 字</span><span className="book-shortcut"><kbd>Ctrl</kbd> + <kbd>Enter</kbd> 确认并继续</span></footer>
    </main>
  );
}

function BookOverview({ project, translatedCount, reviewedCount, issueCount, busy, error, onLanguageChange, onTranslate }: { project: BookProject; translatedCount: number; reviewedCount: number; issueCount: number; busy: boolean; error: string | null; onLanguageChange: (field: "sourceLanguage" | "targetLanguage", code: string) => void; onTranslate: (project: BookProject, chapterId: string | null) => void }) {
  const segmentCount = project.chapters.reduce((count, chapter) => count + chapter.segments.length, 0);
  return <main className="book-stage-page">
    <header><span>OVERVIEW / 项目概览</span><h2>书籍概览</h2><p>先确认书稿结构与进度，再进入翻译和校对。</p></header>
    <BookLanguageControls project={project} onChange={onLanguageChange} />
    <section className="book-overview-hero">
      <div className="book-overview-cover"><strong>{project.title.slice(0, 4)}</strong><span>{project.format.toUpperCase()}</span></div>
      <div><span className="book-stage-kicker">{project.format.toUpperCase()} · 书籍项目</span><h3>《{project.title}》</h3><p>{project.sourcePath}</p><div className="book-overview-progress"><i style={{ width: `${bookProgress(project)}%` }} /></div><b>{bookProgress(project)}% 已完成校对</b><div className="book-overview-actions"><button type="button" className="book-primary-action" disabled={busy} onClick={() => onTranslate(project, null)}>{busy ? "正在翻译全书…" : "一键翻译全书"}</button><span>将按目录顺序翻译全部 {project.chapters.length} 章</span></div>{error ? <span className="book-operation-error" role="alert">{error}</span> : null}</div>
    </section>
    <section className="book-stat-grid" aria-label="书籍统计">
      <article><span>章节</span><strong>{project.chapters.length}</strong><small>已识别目录结构</small></article>
      <article><span>段落</span><strong>{segmentCount}</strong><small>{translatedCount} 段已有译文</small></article>
      <article><span>已校对</span><strong>{reviewedCount}</strong><small>人工确认的段落</small></article>
      <article className={issueCount ? "warning" : ""}><span>待处理</span><strong>{issueCount}</strong><small>质量检查提示</small></article>
    </section>
  </main>;
}

function BookLanguageControls({ project, onChange }: { project: BookProject; onChange: (field: "sourceLanguage" | "targetLanguage", code: string) => void }) {
  return <div className="book-language-controls" aria-label="书籍翻译语言">
    <BookLanguageSelect label="书籍源语言" shortLabel="源语言" value={project.sourceLanguage} options={sourceLanguages} onChange={(code) => onChange("sourceLanguage", code)} />
    <span aria-hidden="true">→</span>
    <BookLanguageSelect label="书籍目标语言" shortLabel="目标语言" value={project.targetLanguage} options={targetLanguages} onChange={(code) => onChange("targetLanguage", code)} />
  </div>;
}

function BookLanguageSelect({ label, shortLabel, value, options, onChange }: { label: string; shortLabel: string; value: string; options: Language[]; onChange: (code: string) => void }) {
  const known = options.some((language) => language.code === value);
  return <label><span>{shortLabel}</span><select aria-label={label} value={value} onChange={(event) => onChange(event.target.value)}>{!known ? <option value={value}>{value}</option> : null}{options.map((language) => <option key={language.code} value={language.code}>{language.name}</option>)}</select></label>;
}

function BookExport({ project, busy, issueCount, onPublicationChange, onExport }: {
  project: BookProject;
  busy: boolean;
  issueCount: number;
  onPublicationChange: (field: keyof PublicationMetadata, value: string) => void;
  onExport: (project: BookProject, format: BookExportFormat, profile: BookExportProfile) => void;
}) {
  const [profile, setProfile] = useState<BookExportProfile>({ printPreset: project.publication.printPreset, includePageNumbers: true, chapterStartsNewPage: true });
  const formats: Array<{ format: BookExportFormat; label: string; title: string; description: string; tag: string }> = [
    { format: "docx", label: "导出 DOCX", title: "出版社可编辑稿", description: "保留书名、章节、正文样式与首行缩进，可交给编辑继续修订。", tag: "推荐交付" },
    { format: "epub", label: "导出 EPUB", title: "EPUB 3 电子书", description: "包含目录、书脊和出版元数据，适合电子阅读器与发行平台。", tag: "电子出版" },
    { format: "pdf", label: "导出 PDF", title: "印刷定稿 PDF", description: "嵌入中文字体，按选定成品尺寸分页，可用于审稿与印前确认。", tag: "印刷预览" },
    { format: "markdown", label: "导出 Markdown", title: "结构化通用书稿", description: "轻量、开放，适合作为长期归档或进入其他排版流程。", tag: "通用归档" },
  ];
  const changePreset = (value: PrintPreset) => {
    setProfile((current) => ({ ...current, printPreset: value }));
    onPublicationChange("printPreset", value);
  };
  return <main className="book-stage-page book-publication-page">
    <header><span>PUBLICATION / 出版交付</span><h2>出版与导出</h2><p>先完善书目信息和印刷版式，再选择交付格式。所有导出都会保留原书与当前项目。</p></header>
    <section className="book-publication-meta" aria-label="出版信息">
      <div className="book-section-heading"><div><span>METADATA</span><h3>书目信息</h3></div><small>会写入 DOCX、EPUB 与 PDF</small></div>
      <div className="book-metadata-grid">
        <BookMetadataField label="作者" value={project.publication.author} onChange={(value) => onPublicationChange("author", value)} />
        <BookMetadataField label="译者" value={project.publication.translator} onChange={(value) => onPublicationChange("translator", value)} />
        <BookMetadataField label="出版社" value={project.publication.publisher} onChange={(value) => onPublicationChange("publisher", value)} />
        <BookMetadataField label="ISBN" value={project.publication.isbn} onChange={(value) => onPublicationChange("isbn", value)} />
        <BookMetadataField label="版权说明" value={project.publication.copyright} onChange={(value) => onPublicationChange("copyright", value)} wide />
      </div>
    </section>
    <section className="book-layout-settings" aria-label="印刷版式">
      <div><span className="book-stage-kicker">PRINT LAYOUT</span><h3>印刷版式</h3><p>PDF 使用此设置；DOCX 同时保留可继续调整的出版样式。</p></div>
      <label><span>成品尺寸</span><select aria-label="印刷成品尺寸" value={profile.printPreset} onChange={(event) => changePreset(event.target.value as PrintPreset)}><option value="large32">大32开 · 140 × 203 mm</option><option value="a5">A5 · 148 × 210 mm</option><option value="sixteen">16开 · 185 × 260 mm</option></select></label>
      <label className="book-check-setting"><input type="checkbox" checked={profile.includePageNumbers} onChange={(event) => setProfile((current) => ({ ...current, includePageNumbers: event.target.checked }))} />显示页码</label>
      <label className="book-check-setting"><input type="checkbox" checked={profile.chapterStartsNewPage} onChange={(event) => setProfile((current) => ({ ...current, chapterStartsNewPage: event.target.checked }))} />章节另起页</label>
    </section>
    <section className="book-export-grid" aria-label="导出格式">
      {formats.map((item) => <article key={item.format} className={`book-export-option ${item.format}`}><span>{item.tag}</span><strong>{item.title}</strong><p>{item.description}</p><button type="button" disabled={busy} onClick={() => onExport(project, item.format, profile)}>{busy ? "正在生成…" : item.label}</button></article>)}
    </section>
    {issueCount > 0 ? <p className="book-export-warning">仍有 {issueCount} 个质量提示；可以导出，但建议先在“校对”阶段处理。</p> : null}
  </main>;
}

function BookMetadataField({ label, value, onChange, wide = false }: { label: string; value: string; onChange: (value: string) => void; wide?: boolean }) {
  return <label className={wide ? "wide" : ""}><span>{label}</span><input value={value} onChange={(event) => onChange(event.target.value)} /></label>;
}

function BookHistory({ project, records, busy, onExport, onOpenExport }: {
  project: BookProject;
  records: BookExportRecord[];
  busy: boolean;
  onExport: (project: BookProject, format: BookExportFormat, profile: BookExportProfile) => void;
  onOpenExport: (path: string) => void;
}) {
  return <main className="book-stage-page book-publication-page"><header><span>HISTORY / 出版记录</span><h2>导出历史</h2><p>这里记录每一次真实生成的书稿文件，可定位文件或按当时的版式再次导出。</p></header>
    {records.length ? <section className="book-history-list" aria-label="书籍导出历史">{records.map((record) => <article key={record.id}><div className={`book-format-badge ${record.format}`}>{record.format.toUpperCase()}</div><div><strong>{record.bookTitle}</strong><p>{record.outputPath}</p><span>{formatExportTime(record.exportedAtUnixMs)} · {printPresetLabel(record.profile.printPreset)} · {record.targetLanguage}</span></div><div className="book-history-actions"><button type="button" onClick={() => onOpenExport(record.outputPath)}>在文件夹中显示</button><button type="button" disabled={busy} onClick={() => onExport(project, record.format, record.profile)}>按此设置再次导出</button></div></article>)}</section> : <section className="book-history-empty"><strong>还没有导出记录</strong><p>{project.sourcePath}</p><span>完成第一次 DOCX、EPUB、PDF 或 Markdown 导出后，记录会出现在这里。</span></section>}
  </main>;
}

function formatExportTime(value: number) {
  return new Intl.DateTimeFormat("zh-CN", { year: "numeric", month: "2-digit", day: "2-digit", hour: "2-digit", minute: "2-digit" }).format(new Date(value));
}

function printPresetLabel(value: PrintPreset) {
  return value === "large32" ? "大32开" : value === "a5" ? "A5" : "16开";
}

function ChapterTree({ project, chapterId, onSelect }: { project: BookProject; chapterId: string; onSelect: (chapter: BookChapter) => void }) {
  return <aside className="book-chapters" aria-label="章节目录">
    <div className="book-panel-heading"><div><span>CONTENTS</span><h2>书稿目录</h2></div><small>{project.chapters.length} 章</small></div>
    <div className="book-progress"><span>全书进度</span><strong>{bookProgress(project)}%</strong><div><i style={{ width: `${bookProgress(project)}%` }} /></div></div>
    <ol>{project.chapters.map((chapter, index) => {
      const reviewed = chapter.segments.filter((segment) => segment.status === "reviewed").length;
      const progress = chapter.segments.length ? Math.round((reviewed / chapter.segments.length) * 100) : 0;
      const issues = chapter.segments.filter((segment) => segment.status === "issue").length;
      return <li key={chapter.id}><button type="button" className={chapter.id === chapterId ? "active" : ""} aria-current={chapter.id === chapterId ? "page" : undefined} onClick={() => onSelect(chapter)}><span className="book-chapter-number">{String(index + 1).padStart(2, "0")}</span><span><strong>{chapter.title}</strong><small>{progress}% 已校对{issues ? ` · ${issues} 个问题` : ""}</small></span>{issues > 0 && <b>{issues}</b>}</button></li>;
    })}</ol>
  </aside>;
}

type EditorProps = { segments: BookSegment[]; selectedId: string; onSelect: (id: string) => void; onChange: (id: string, value: string) => void; onConfirm: (id: string) => void };

function ReadingEditor({ chapter, chapterIndex, selectedId, onSelect, onChange, onConfirm }: Omit<EditorProps, "segments"> & { chapter: BookChapter; chapterIndex: number }) {
  return <section className="book-reading-page" aria-label="阅读编辑"><header><span>CHAPTER {String(chapterIndex + 1).padStart(2, "0")}</span><h2>{chapter.title}</h2><p>第 {chapterIndex + 1} 章</p></header><div className="book-manuscript">{chapter.segments.map((segment, index) => <article key={segment.id} className={`book-paragraph ${selectedId === segment.id ? "selected" : ""}`}><button type="button" className={`book-status-dot ${segment.status}`} aria-label={`选择段落 ${index + 1}`} aria-current={selectedId === segment.id ? "true" : undefined} onClick={() => onSelect(segment.id)} /><p className="book-source-preview">{segment.source}</p><textarea aria-label={`段落 ${index + 1} 译文`} value={segment.translation} placeholder="等待翻译，或在此输入译文…" rows={Math.max(2, Math.ceil(Math.max(segment.translation.length, 20) / 35))} onFocus={() => onSelect(segment.id)} onChange={(event) => onChange(segment.id, event.target.value)} onKeyDown={(event) => handleConfirm(event, segment.id, onConfirm)} /></article>)}</div></section>;
}

function SegmentReview({ segments, selectedId, onSelect, onChange, onConfirm }: EditorProps) {
  return <section className="book-segment-review" aria-label="逐段校对"><div className="book-review-columns"><span>原文</span><span>译文</span></div>{segments.map((segment, index) => <div key={segment.id} className={`book-review-row ${selectedId === segment.id ? "selected" : ""}`}><button type="button" className="book-row-index" aria-label={`选择段落 ${index + 1}`} aria-current={selectedId === segment.id ? "true" : undefined} onClick={() => onSelect(segment.id)}>{String(index + 1).padStart(2, "0")}</button><span className="book-source-cell">{segment.source}</span><textarea aria-label={`段落 ${index + 1} 译文`} value={segment.translation} placeholder="等待翻译…" onFocus={() => onSelect(segment.id)} onChange={(event) => onChange(segment.id, event.target.value)} onKeyDown={(event) => handleConfirm(event, segment.id, onConfirm)} /></div>)}</section>;
}

function Inspector({ segment, index, providerName }: { segment: BookSegment; index: number; providerName: string | null }) {
  const labels = { untranslated: "待翻译", draft: "AI 草稿", reviewed: "已校对", issue: "有问题" };
  return <aside className="book-inspector" aria-label="段落检查器"><div className="book-panel-heading"><div><span>INSPECTOR</span><h2>当前段落 {index + 1}</h2></div><b className={segment.status}>{labels[segment.status]}</b></div><section><h3>原文</h3><p className="book-inspector-source">{segment.source}</p></section><section><h3>质量检查</h3>{segment.qaNote ? <div className="book-qa-note"><strong>发现待处理问题</strong><p>{segment.qaNote}</p></div> : <div className="book-qa-clear"><strong>未发现阻断问题</strong><p>{segment.status === "untranslated" ? "翻译后将自动执行质量检查。" : "基础完整性检查已通过。"}</p></div>}</section><section><h3>命中术语</h3>{segment.terms.length ? segment.terms.map((term) => <span className="book-term-chip" key={term}>{term}</span>) : <p className="book-muted">当前段落没有命中术语。</p>}</section><section className="book-ai-suggestion"><div><h3>模型</h3><span>{providerName ?? "未配置"}</span></div><p>翻译结果会保存到独立项目文件，不会覆盖原书。</p></section></aside>;
}

function handleConfirm(event: KeyboardEvent<HTMLTextAreaElement>, id: string, onConfirm: (id: string) => void) {
  if (event.ctrlKey && event.key === "Enter") { event.preventDefault(); onConfirm(id); }
}
