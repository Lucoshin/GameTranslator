export type PatchHistoryEntry = {
  id: string;
  projectPath: string;
  patchPath: string;
  targetLanguage: string;
  fileCount: number;
  exportedAtUnixMs: number;
  installedAtUnixMs: number | null;
};

export function PatchHistory({
  entries,
  loading,
  error,
  notice,
  uninstallingId,
  installingId,
  deletingId,
  onInstall,
  onUninstall,
  onDelete,
}: {
  entries: PatchHistoryEntry[];
  loading: boolean;
  error: string | null;
  notice: string | null;
  uninstallingId: string | null;
  installingId: string | null;
  deletingId: string | null;
  onInstall: (id: string) => void;
  onUninstall: (id: string) => void;
  onDelete: (id: string) => void;
}) {
  return (
    <div className="page history-page">
      <section className="page-heading compact">
        <div>
          <p className="kicker">PATCH HISTORY</p>
          <h1>补丁历史</h1>
          <p className="muted">仅管理由 GameTranslator 导出或安装的补丁</p>
        </div>
        <span className="stamp-status small">{loading ? "读取中" : `${entries.length} 项`}</span>
      </section>
      {error ? <div className="notice-banner" role="alert"><span>ERROR</span>{error}</div> : null}
      {notice ? <div className="notice-banner"><span>DONE</span>{notice}</div> : null}
      {loading ? <section className="panel history-empty">正在读取补丁历史…</section> : null}
      {!loading && !entries.length ? <section className="panel history-empty"><b>还没有补丁记录</b><p>完成补丁导出后，记录会保存在此处；已安装的 Ren'Py 或 RimWorld 语言包可从这里安全卸载。</p></section> : null}
      {!loading && entries.length ? <section className="history-list" aria-label="补丁历史列表">
        {entries.map((entry) => {
          const installed = entry.installedAtUnixMs !== null;
          return <article className="history-entry" key={entry.id}>
            <div className="history-entry-main">
              <div className="history-entry-title"><b>{fileName(entry.patchPath)}</b><span className={installed ? "history-status installed" : "history-status"}>{installed ? "已安装" : "已导出"}</span></div>
              <p>{fileName(entry.projectPath)} · {entry.targetLanguage} · {entry.fileCount} 个文件 · {formatTime(entry.installedAtUnixMs ?? entry.exportedAtUnixMs)}</p>
              <code title={entry.patchPath}>{entry.patchPath}</code>
            </div>
            <div className="history-actions">
              {installed ? <button className="secondary-action" aria-label="卸载翻译补丁" disabled={uninstallingId !== null || installingId !== null} onClick={() => onUninstall(entry.id)}>{uninstallingId === entry.id ? "正在卸载…" : "卸载翻译补丁"}</button> : <button className="secondary-action" aria-label="安装到当前内容" disabled={installingId !== null || uninstallingId !== null} onClick={() => onInstall(entry.id)}>{installingId === entry.id ? "正在安装…" : "安装到当前内容"}</button>}
              <button className="history-delete" aria-label="删除历史记录" disabled={installed || deletingId !== null || installingId !== null || uninstallingId !== null} title={installed ? "请先卸载补丁" : "仅删除历史记录，不删除补丁文件"} onClick={() => onDelete(entry.id)}>{deletingId === entry.id ? "正在删除…" : "删除记录"}</button>
            </div>
          </article>;
        })}
      </section> : null}
      <section className="safety-callout history-safety"><span>安全</span><div><b>卸载会优先恢复安装前备份</b><p>没有备份时，仅删除仍与已安装补丁完全一致的文件；发现外部修改会停止，不会覆盖用户内容。</p></div></section>
    </div>
  );
}

function fileName(path: string) {
  return path.split(/[\\/]/).at(-1) ?? path;
}

function formatTime(timestamp: number) {
  return new Date(timestamp).toLocaleString("zh-CN", {
    dateStyle: "medium",
    timeStyle: "short",
  });
}
