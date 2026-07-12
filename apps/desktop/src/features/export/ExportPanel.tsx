import type { Language } from "../translation/LanguageSettings";

type ExportResult = { outputPath: string; fileCount: number };

export function ExportPanel({
  demo,
  result,
  error,
  exporting,
  canExport,
  onExport,
  targetLanguage,
}: {
  demo: boolean;
  result: ExportResult | null;
  error: string | null;
  exporting: boolean;
  canExport: boolean;
  onExport: () => void;
  targetLanguage: Language;
}) {
  const fileCount = demo ? 24 : (result?.fileCount ?? 0);
  return (
    <div className="page export-page">
      <section className="page-heading compact"><div><p className="kicker">PATCH BUILDER</p><h1>导出汉化补丁</h1><p className="muted">原游戏文件不会被直接修改</p></div><span className="stamp-status small">{error ? "生成失败" : result ? "校验通过" : exporting ? "正在校验" : "等待生成"}</span></section>
      {error ? <div className="demo-banner" role="alert"><span>ERROR</span>{error}</div> : null}
      {result ? <div className="demo-banner"><span>DONE</span><b>{result.outputPath}</b></div> : null}
      <div className="export-layout">
        <section className="panel export-summary">
          <div className="panel-title"><span>导出清单</span><small>PATCH MANIFEST</small></div>
          <div className="file-stack"><i/><i/><i/><div><strong>{fileCount}</strong><span>个翻译文件</span></div></div>
          <dl><div><dt>补丁格式</dt><dd>GameTranslator Patch v1</dd></div><div><dt>目标语言</dt><dd>{targetLanguage.name} {targetLanguage.code}</dd></div><div><dt>完整性</dt><dd className={result ? "safe" : ""}>{result ? "SHA-256 已校验" : "生成时执行 SHA-256 校验"}</dd></div></dl>
        </section>
        <section className="panel export-action">
          <div className="safety-callout"><span>只读</span><div><b>原始游戏保持不变</b><p>补丁将写入新的目录，并附带文件哈希与恢复说明。</p></div></div>
          <p className="muted">点击生成后选择父目录，应用会创建独立的 <b>游戏名-{targetLanguage.code}</b> 文件夹。</p>
          <button className="primary-action full" aria-label="生成汉化补丁" disabled={!canExport || exporting} onClick={onExport}>{exporting ? "正在生成…" : "生成汉化补丁"} <span>↗</span></button>
        </section>
      </div>
    </div>
  );
}
