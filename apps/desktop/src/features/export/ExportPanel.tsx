import type { Language } from "../translation/LanguageSettings";

type ExportResult = { outputPath: string; fileCount: number };
type InstallResult = { installedPath: string; fileCount: number };

export function ExportPanel({
  result,
  error,
  exporting,
  installing,
  canExport,
  onExport,
  onInstall,
  targetLanguage,
  installResult,
  installError,
}: {
  result: ExportResult | null;
  error: string | null;
  exporting: boolean;
  installing: boolean;
  canExport: boolean;
  onExport: () => void;
  onInstall: () => void;
  targetLanguage: Language;
  installResult: InstallResult | null;
  installError: string | null;
}) {
  const fileCount = result?.fileCount ?? 0;
  return (
    <div className="page export-page">
      <section className="page-heading compact"><div><p className="kicker">PATCH BUILDER</p><h1>导出翻译补丁</h1><p className="muted">先独立导出并校验，再选择是否安装到原内容</p></div><span className="stamp-status small">{error || installError ? "操作失败" : installResult ? "已安装" : result ? "校验通过" : exporting ? "正在校验" : "等待生成"}</span></section>
      {error ? <div className="notice-banner" role="alert"><span>ERROR</span>{error}</div> : null}
      {installError ? <div className="notice-banner" role="alert"><span>ERROR</span>{installError}</div> : null}
      {result ? <div className="notice-banner"><span>DONE</span><b>{result.outputPath}</b></div> : null}
      {installResult ? <div className="notice-banner"><span>INSTALLED</span><b>翻译已安装，重新启动游戏后生效</b></div> : null}
      <div className="export-layout">
        <section className="panel export-summary">
          <div className="panel-title"><span>导出清单</span><small>PATCH MANIFEST</small></div>
          <div className="file-stack"><i/><i/><i/><div><strong>{fileCount}</strong><span>个翻译文件</span></div></div>
          <dl><div><dt>补丁格式</dt><dd>GameTranslator Patch v1</dd></div><div><dt>目标语言</dt><dd>{targetLanguage.name} {targetLanguage.code}</dd></div><div><dt>完整性</dt><dd className={result ? "safe" : ""}>{result ? "SHA-256 已校验" : "生成时执行 SHA-256 校验"}</dd></div></dl>
        </section>
        <section className="panel export-action">
          <div className="safety-callout"><span>安全</span><div><b>导出阶段不修改游戏</b><p>安装前校验所有文件；冲突文件会备份到补丁目录。</p></div></div>
          <p className="muted">先生成独立的 <b>内容名-{targetLanguage.code}</b> 目录。Ren'Py 与 RimWorld 模组可在校验后直接安装。</p>
          <button className="primary-action full" aria-label="生成翻译补丁" disabled={!canExport || exporting} onClick={onExport}>{exporting ? "正在生成…" : "生成翻译补丁"} <span>↗</span></button>
          {result ? <button className="secondary-action full" aria-label="安装到当前内容" disabled={installing} onClick={onInstall}>{installing ? "正在安装…" : `安装${targetLanguage.name}翻译到当前内容`}</button> : null}
        </section>
      </div>
    </div>
  );
}
