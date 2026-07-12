import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ExportPanel } from "./features/export/ExportPanel";
import { ProjectHome } from "./features/projects/ProjectHome";
import { ProjectOverview, type ProjectSummary } from "./features/projects/ProjectOverview";
import { ProviderDrawer, type ProviderConfiguration } from "./features/providers/ProviderDrawer";
import { PatchHistory, type PatchHistoryEntry } from "./features/history/PatchHistory";
import { SegmentTable } from "./features/review/SegmentTable";
import { TranslationProgress } from "./features/translation/TranslationProgress";
import { LanguageSettings, type Language } from "./features/translation/LanguageSettings";
import "./styles/global.css";

type View = "overview" | "translation" | "review" | "export" | "history";
export type TranslationItem = { id: string; source: string; target: string; speaker: string | null; sourceFile: string; qa: "passed" | "warning" | "blocking" };
export type TranslationRun = { items: TranslationItem[]; warningFindings: number; blockingFindings: number; failedSegmentIds: string[] };
export type TranslationProgressState = { phase: "idle" | "extracting" | "translating" | "qa" | "completed" | "failed"; completed: number; total: number; failed: number; warningFindings: number; blockingFindings: number; message: string; concurrency?: number; throughput?: number; etaSeconds?: number };
export type TranslationLog = { time: string; message: string };
type TranslationProgressEvent = TranslationProgressState & { runId: string };
type ExportResult = { outputPath: string; fileCount: number };
type InstallResult = { installedPath: string; fileCount: number };
type UninstallResult = { restoredFileCount: number; removedFileCount: number };
type DesktopPreferences = { recentProjectPath: string | null; sourceLanguage: Language; targetLanguage: Language };

export default function App() {
  const [project, setProject] = useState<ProjectSummary | null>(null);
  const [openError, setOpenError] = useState<string | null>(null);
  const [scanning, setScanning] = useState(false);
  const [view, setView] = useState<View>("overview");
  const [homeOpen, setHomeOpen] = useState(false);
  const [overviewOpen, setOverviewOpen] = useState(false);
  const [providerOpen, setProviderOpen] = useState(false);
  const [provider, setProvider] = useState<ProviderConfiguration | null>(null);
  const [persistenceError, setPersistenceError] = useState<string | null>(null);
  const [sourceLanguage, setSourceLanguage] = useState<Language>({ code: "auto", name: "自动检测" });
  const [targetLanguage, setTargetLanguage] = useState<Language>({ code: "zh-CN", name: "简体中文" });
  const [translation, setTranslation] = useState<TranslationRun | null>(null);
  const [translationError, setTranslationError] = useState<string | null>(null);
  const [translating, setTranslating] = useState(false);
  const [progress, setProgress] = useState<TranslationProgressState | null>(null);
  const [translationLogs, setTranslationLogs] = useState<TranslationLog[]>([]);
  const [exportResult, setExportResult] = useState<ExportResult | null>(null);
  const [exportError, setExportError] = useState<string | null>(null);
  const [exporting, setExporting] = useState(false);
  const [installResult, setInstallResult] = useState<InstallResult | null>(null);
  const [installError, setInstallError] = useState<string | null>(null);
  const [installing, setInstalling] = useState(false);
  const [history, setHistory] = useState<PatchHistoryEntry[]>([]);
  const [historyError, setHistoryError] = useState<string | null>(null);
  const [historyNotice, setHistoryNotice] = useState<string | null>(null);
  const [historyLoading, setHistoryLoading] = useState(false);
  const [uninstallingId, setUninstallingId] = useState<string | null>(null);
  const [installingHistoryId, setInstallingHistoryId] = useState<string | null>(null);
  const [deletingHistoryId, setDeletingHistoryId] = useState<string | null>(null);

  useEffect(() => {
    void invoke<ProviderConfiguration | null>("load_provider_configuration")
      .then((stored) => { if (stored) setProvider(stored); })
      .catch((error: unknown) => setPersistenceError(`读取模型配置失败：${String(error)}`));
    void invoke<DesktopPreferences>("load_desktop_preferences")
      .then((preferences) => {
        if (!preferences) return;
        setSourceLanguage(preferences.sourceLanguage);
        setTargetLanguage(preferences.targetLanguage);
        if (!preferences.recentProjectPath) return;
        return invoke<ProjectSummary>("scan_project_path", { projectPath: preferences.recentProjectPath })
          .then((restored) => {
            setProject(restored);
            setOverviewOpen(true);
          })
          .catch((error: unknown) => setPersistenceError(`无法恢复最近项目：${String(error)}`));
      })
      .catch((error: unknown) => setPersistenceError(`读取应用偏好失败：${String(error)}`));
  }, []);

  const savePreferences = (recentProjectPath: string | null, source = sourceLanguage, target = targetLanguage) => {
    void invoke("save_desktop_preferences", { preferences: { recentProjectPath, sourceLanguage: source, targetLanguage: target } })
      .catch((error: unknown) => setPersistenceError(`保存应用偏好失败：${String(error)}`));
  };

  const saveProvider = (configuration: ProviderConfiguration) => {
    void invoke("save_provider_configuration", { provider: configuration })
      .then(() => {
        const persisted = { ...configuration, apiKey: undefined };
        setProvider(persisted);
        setProviderOpen(false);
      })
      .catch((error: unknown) => setTranslationError(String(error)));
  };

  const selectProject = () => {
    setOpenError(null);
    setScanning(true);
    void invoke<ProjectSummary>("select_and_scan_project")
      .then((result) => { setProject(result); savePreferences(result.projectPath); setHistory([]); setHistoryError(null); setHistoryNotice(null); setHomeOpen(false); setOverviewOpen(true); setView("overview"); })
      .catch((error: unknown) => setOpenError(String(error)))
      .finally(() => setScanning(false));
  };

  if (homeOpen || (!project && !overviewOpen)) {
    return <>
      <ProjectHome
        error={openError ?? persistenceError}
        scanning={scanning}
        providerName={provider?.model ?? null}
        onConfigure={() => setProviderOpen(true)}
        onSelect={selectProject}
        onEnterOverview={() => { setHomeOpen(false); setOverviewOpen(true); setView("overview"); }}
        onReturn={project ? () => { setHomeOpen(false); setView("overview"); } : undefined}
      />
      <ProviderDrawer open={providerOpen} current={provider} onClose={() => setProviderOpen(false)} onSave={saveProvider} />
    </>;
  }

  const startTranslation = () => {
    if (!project) return;
    if (!provider) {
      setProviderOpen(true);
      return;
    }
    setTranslationError(null);
    setTranslation(null);
    setTranslating(true);
    setView("translation");
    const runId = `${Date.now()}-${Math.random().toString(16).slice(2)}`;
    const initial: TranslationProgressState = { phase: "extracting", completed: 0, total: 0, failed: 0, warningFindings: 0, blockingFindings: 0, message: "正在准备翻译任务" };
    setProgress(initial);
    setTranslationLogs([{ time: currentTime(), message: initial.message }]);
    void (async () => {
      let active = true;
      let receivedBackendEvent = false;
      let unlisten: (() => void) | undefined;
      void listen<TranslationProgressEvent>("translation-progress", ({ payload }) => {
        if (payload.runId !== runId) return;
        receivedBackendEvent = true;
        setProgress(payload);
        setTranslationLogs((current) => [...current.slice(-99), { time: currentTime(), message: payload.message }]);
      }).then((cleanup) => {
        if (active) unlisten = cleanup;
        else cleanup();
      }).catch((error: unknown) => {
        setTranslationLogs((current) => [...current, { time: currentTime(), message: `进度监听不可用：${String(error)}` }]);
      });
      const startupWatchdog = window.setTimeout(() => {
        if (receivedBackendEvent) return;
        const message = "后端启动超过 15 秒，任务仍在尝试运行";
        setProgress((current) => ({ ...(current ?? initial), message }));
        setTranslationLogs((current) => [...current, { time: currentTime(), message }]);
      }, 15_000);
      try {
        const result = await invoke<TranslationRun>("translate_project", {
          input: {
            runId,
            projectPath: project.projectPath,
            provider: { ...provider, apiKey: null },
            sourceLanguage,
            targetLanguage,
          },
        });
        setTranslation(result);
        setProgress((current) => {
          const total = current?.total || result.items.length;
          return {
            phase: "completed",
            completed: total,
            total,
            failed: result.failedSegmentIds.length,
            warningFindings: result.warningFindings,
            blockingFindings: result.blockingFindings,
            message: "任务完成，可以校对或导出",
          };
        });
        setTranslationLogs((current) => current.at(-1)?.message === "任务完成，可以校对或导出" ? current : [...current, { time: currentTime(), message: "任务完成，可以校对或导出" }]);
      } catch (error) {
        const message = String(error);
        setTranslationError(message);
        setProgress((current) => ({ ...(current ?? initial), phase: "failed", message }));
        setTranslationLogs((current) => [...current, { time: currentTime(), message: `任务失败：${message}` }]);
      } finally {
        active = false;
        window.clearTimeout(startupWatchdog);
        unlisten?.();
        setTranslating(false);
      }
    })();
  };

  const exportPatch = () => {
    if (!project || !translation) {
      setExportError("没有可导出的翻译结果");
      return;
    }
    setExportError(null);
    setInstallResult(null);
    setInstallError(null);
    setExporting(true);
    void invoke<ExportResult>("export_translation_patch", {
      input: { projectPath: project.projectPath, items: translation.items, targetLanguage },
    })
      .then(setExportResult)
      .catch((error: unknown) => setExportError(String(error)))
      .finally(() => setExporting(false));
  };

  const installPatch = () => {
    if (!project || !exportResult) return;
    setInstalling(true);
    setInstallError(null);
    void invoke<InstallResult>("install_translation_patch", {
      input: {
        projectPath: project.projectPath,
        patchPath: exportResult.outputPath,
        targetLanguage,
      },
    })
      .then(setInstallResult)
      .catch((error: unknown) => setInstallError(String(error)))
      .finally(() => setInstalling(false));
  };

  const openHistory = () => {
    setView("history");
    setHistoryError(null);
    setHistoryNotice(null);
    setHistoryLoading(true);
    void invoke<PatchHistoryEntry[]>("list_patch_history", { projectPath: project?.projectPath ?? null })
      .then(setHistory)
      .catch((error: unknown) => setHistoryError(String(error)))
      .finally(() => setHistoryLoading(false));
  };

  const uninstallPatch = (id: string) => {
    const entry = history.find((candidate) => candidate.id === id);
    if (!entry) return;
    setHistoryError(null);
    setHistoryNotice(null);
    setUninstallingId(id);
    void invoke<UninstallResult>("uninstall_translation_patch", {
      input: { projectPath: entry.projectPath, id },
    })
      .then((result) => {
        setHistory((entries) => entries.map((entry) => entry.id === id ? { ...entry, installedAtUnixMs: null } : entry));
        setHistoryNotice("已卸载");
      })
      .catch((error: unknown) => setHistoryError(String(error)))
      .finally(() => setUninstallingId(null));
  };

  const installHistoryPatch = (id: string) => {
    const entry = history.find((candidate) => candidate.id === id);
    if (!entry) return;
    setHistoryError(null);
    setHistoryNotice(null);
    setInstallingHistoryId(id);
    void invoke<InstallResult>("install_translation_patch", {
      input: { projectPath: entry.projectPath, patchPath: entry.patchPath, targetLanguage: { code: entry.targetLanguage, name: entry.targetLanguage } },
    })
      .then(() => {
        setHistory((entries) => entries.map((candidate) => candidate.id === id ? { ...candidate, installedAtUnixMs: Date.now() } : candidate));
        setHistoryNotice("已安装");
      })
      .catch((error: unknown) => setHistoryError(String(error)))
      .finally(() => setInstallingHistoryId(null));
  };

  const deleteHistoryEntry = (id: string) => {
    const entry = history.find((candidate) => candidate.id === id);
    if (!entry) return;
    setHistoryError(null);
    setHistoryNotice(null);
    setDeletingHistoryId(id);
    void invoke("delete_patch_history_entry", { input: { projectPath: entry.projectPath, id } })
      .then(() => {
        setHistory((entries) => entries.filter((entry) => entry.id !== id));
        setHistoryNotice("历史记录已删除，补丁文件仍保留在原位置");
      })
      .catch((error: unknown) => setHistoryError(String(error)))
      .finally(() => setDeletingHistoryId(null));
  };

  return (
    <div className="app-shell">
      <aside className="rail">
        <button className="brand-mark" onClick={() => setHomeOpen(true)} aria-label="返回全局首页">
          <span>译</span>
        </button>
        <nav aria-label="项目导航">
          <RailButton label="概览" active={view === "overview"} onClick={() => setView("overview")} icon="overview" />
          <RailButton label="任务" active={view === "translation"} disabled={!project} onClick={() => setView("translation")} icon="task" />
          <RailButton label="校对" active={view === "review"} disabled={!project} onClick={() => setView("review")} icon="review" />
          <RailButton label="导出" active={view === "export"} disabled={!project} onClick={() => setView("export")} icon="export" />
          <RailButton label="历史" active={view === "history"} onClick={openHistory} icon="history" />
        </nav>
        <button className="rail-exit" onClick={() => { setProject(null); setOverviewOpen(false); savePreferences(null); }} aria-label="关闭项目">
          ×
        </button>
      </aside>

      <div className="workspace">
        <header className="topbar">
          <div>
            <span className="eyebrow">PROJECT / 本地项目</span>
            <strong>{project?.projectName ?? "尚未选择项目"}</strong>
          </div>
          <div className="topbar-actions">
            <span className="model-chip"><i />{provider?.model ?? "未配置"}</span>
            <button className="text-button" aria-label="顶部配置模型" onClick={() => setProviderOpen(true)}>配置模型</button>
          </div>
        </header>

        <main className="content-stage">
          {persistenceError ? <div className="notice-banner" role="alert"><span>ERROR</span>{persistenceError}</div> : null}
          {view === "overview" && project ? <LanguageSettings source={sourceLanguage} target={targetLanguage} onSourceChange={(language) => { setSourceLanguage(language); savePreferences(project.projectPath, language, targetLanguage); }} onTargetChange={(language) => { setTargetLanguage(language); savePreferences(project.projectPath, sourceLanguage, language); }} /> : null}
          {view === "overview" ? (
            <ProjectOverview
              project={project}
              configured={provider !== null}
              onConfigure={() => setProviderOpen(true)}
              onSelect={selectProject}
              scanning={scanning}
              onStart={startTranslation}
            />
          ) : null}
          {view === "translation" && project ? (
            <TranslationProgress
              result={translation}
              loading={translating}
              error={translationError}
              progress={progress}
              logs={translationLogs}
              sourceCount={project.previewItems?.length ?? project.segmentCount}
              onStart={startTranslation}
              onReview={() => setView("review")}
              onExport={() => setView("export")}
            />
          ) : null}
          {view === "review" && project ? (
            <SegmentTable
              items={translation?.items ?? project.previewItems ?? []}
              translated={translation !== null}
              onApply={(changes) => setTranslation((current) => current ? {
                ...current,
                items: current.items.map((item) => {
                  const change = changes.find((candidate) => candidate.id === item.id);
                  return change ? { ...item, target: change.target } : item;
                }),
              } : current)}
              onExport={() => setView("export")}
              targetLanguage={targetLanguage}
              sourceLanguage={sourceLanguage}
            />
          ) : null}
          {view === "export" && project ? (
            <ExportPanel
              result={exportResult}
              error={exportError}
              exporting={exporting}
              installing={installing}
              canExport={translation !== null}
              onExport={exportPatch}
              onInstall={installPatch}
              targetLanguage={targetLanguage}
              installResult={installResult}
              installError={installError}
            />
          ) : null}
          {view === "history" ? <PatchHistory entries={history} loading={historyLoading} error={historyError} notice={historyNotice} uninstallingId={uninstallingId} installingId={installingHistoryId} deletingId={deletingHistoryId} onInstall={installHistoryPatch} onUninstall={uninstallPatch} onDelete={deleteHistoryEntry} /> : null}
        </main>
      </div>

      <ProviderDrawer
        open={providerOpen}
        current={provider}
        onClose={() => setProviderOpen(false)}
        onSave={saveProvider}
      />
    </div>
  );
}

function currentTime() {
  return new Date().toLocaleTimeString("zh-CN", { hour12: false });
}

function RailButton({
  label,
  icon,
  active,
  disabled = false,
  onClick,
}: {
  label: string;
  icon: "overview" | "task" | "review" | "export" | "history";
  active: boolean;
  disabled?: boolean;
  onClick: () => void;
}) {
  return (
    <button className={active ? "rail-button active" : "rail-button"} disabled={disabled} onClick={onClick} aria-label={label}>
      <RailIcon icon={icon} />
      <small>{label}</small>
    </button>
  );
}

function RailIcon({ icon }: { icon: "overview" | "task" | "review" | "export" | "history" }) {
  const path = icon === "overview" ? <><rect x="3" y="3" width="7" height="7" /><rect x="14" y="3" width="7" height="7" /><rect x="3" y="14" width="7" height="7" /><rect x="14" y="14" width="7" height="7" /></>
    : icon === "task" ? <><circle cx="12" cy="12" r="8.5" /><path d="m10 8 6 4-6 4z" /></>
      : icon === "review" ? <><path d="m4 17.5.7-4.2L15.8 2.2l4 4L8.7 17.3z" /><path d="m13.4 4.6 4 4" /><path d="M4 21h16" /></>
        : icon === "export" ? <><path d="M12 3v11" /><path d="m8 10 4 4 4-4" /><path d="M4 15v5h16v-5" /></>
          : <><circle cx="12" cy="12" r="8.5" /><path d="M12 7v5l3.5 2" /></>;
  return <svg className="rail-icon" viewBox="0 0 24 24" aria-hidden="true">{path}</svg>;
}
