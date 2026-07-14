import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ExportPanel } from "./features/export/ExportPanel";
import { ProjectOverview, type ProjectSummary } from "./features/projects/ProjectOverview";
import { ProviderDrawer, type ProviderConfiguration } from "./features/providers/ProviderDrawer";
import { PatchHistory, type PatchHistoryEntry } from "./features/history/PatchHistory";
import { SegmentTable } from "./features/review/SegmentTable";
import { TranslationProgress } from "./features/translation/TranslationProgress";
import { LanguageSettings, sourceLanguages, targetLanguages, type Language } from "./features/translation/LanguageSettings";
import { BookWorkspace } from "./features/books/BookWorkspace";
import { ProjectCenter } from "./features/books/ProjectCenter";
import type { BookExportFormat, BookExportProfile, BookExportRecord, BookProject } from "./features/books/contracts";
import { WorkspaceChrome, type WorkspaceStage } from "./features/workspace/WorkspaceChrome";
import { applicationErrorMessage } from "./api/errors";
import type { ResumableTask, TranslationItem, TranslationProgressEvent, TranslationProgressState, TranslationRun } from "./api/contracts";
import "./styles/global.css";

type View = "overview" | "translation" | "review" | "export" | "history";
export type { TranslationItem, TranslationRun, TranslationProgressState } from "./api/contracts";
export type TranslationLog = { time: string; message: string };
type ExportResult = { outputPath: string; fileCount: number };
type InstallResult = { installedPath: string; fileCount: number };
type UninstallResult = { restoredFileCount: number; removedFileCount: number };
type DesktopPreferences = { recentProjectPath: string | null; sourceLanguage: Language; targetLanguage: Language };

export default function App() {
  const [workspaceKind, setWorkspaceKind] = useState<"game" | "library" | "book">("library");
  const [bookProjects, setBookProjects] = useState<BookProject[]>([]);
  const [activeBook, setActiveBook] = useState<BookProject | null>(null);
  const [bookLoading, setBookLoading] = useState(false);
  const [bookBusy, setBookBusy] = useState(false);
  const [bookError, setBookError] = useState<string | null>(null);
  const [bookExportHistory, setBookExportHistory] = useState<BookExportRecord[]>([]);
  const [project, setProject] = useState<ProjectSummary | null>(null);
  const [openError, setOpenError] = useState<string | null>(null);
  const [scanning, setScanning] = useState(false);
  const [view, setView] = useState<View>("overview");
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
    void invoke<BookProject[]>("list_book_projects")
      .then((projects) => setBookProjects(projects ?? []))
      .catch((error: unknown) => setBookError(`读取书籍项目失败：${applicationErrorMessage(error)}`));
    void invoke<ResumableTask[]>("list_resumable_tasks")
      .then((tasks) => {
        const latest = tasks?.at(0);
        if (!latest) return;
        setPersistenceError(`检测到未完成任务（${latest.completed}/${latest.total}），再次开始翻译将从缓存继续`);
        return invoke<ProjectSummary>("scan_project_path", { projectPath: latest.projectPath })
          .then((restored) => { setProject(restored); setWorkspaceKind("game"); });
      })
      .catch((error: unknown) => setPersistenceError(`读取恢复任务失败：${applicationErrorMessage(error)}`));
    void invoke<ProviderConfiguration | null>("load_provider_configuration")
      .then((stored) => { if (stored) setProvider(stored); })
      .catch((error: unknown) => setPersistenceError(`读取模型配置失败：${applicationErrorMessage(error)}`));
    void invoke<DesktopPreferences>("load_desktop_preferences")
      .then((preferences) => {
        if (!preferences) return;
        setSourceLanguage(preferences.sourceLanguage);
        setTargetLanguage(preferences.targetLanguage);
        if (!preferences.recentProjectPath) return;
        return invoke<ProjectSummary>("scan_project_path", { projectPath: preferences.recentProjectPath })
          .then((restored) => {
            setProject(restored);
            setWorkspaceKind("game");
          })
          .catch((error: unknown) => setPersistenceError(`无法恢复最近项目：${applicationErrorMessage(error)}`));
      })
      .catch((error: unknown) => setPersistenceError(`读取应用偏好失败：${applicationErrorMessage(error)}`));
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

  const closeBookWorkspace = () => {
    window.location.hash = "projects";
    setWorkspaceKind("library");
  };

  const openBook = (book: BookProject) => {
    setActiveBook(book);
    setBookError(null);
    window.location.hash = "book";
    setView("translation");
    setWorkspaceKind("book");
    void loadBookExportHistory(book.id);
  };

  const loadBookExportHistory = (projectId: string) => invoke<BookExportRecord[]>("list_book_export_history", { projectId })
    .then((records) => setBookExportHistory(records ?? []))
    .catch((error: unknown) => setBookError(`读取书籍导出历史失败：${applicationErrorMessage(error)}`));

  const importBook = () => {
    setBookLoading(true);
    setBookError(null);
    void invoke<BookProject>("import_book_project")
      .then((book) => {
        setBookProjects((current) => [book, ...current.filter((item) => item.id !== book.id)]);
        openBook(book);
      })
      .catch((error: unknown) => setBookError(applicationErrorMessage(error)))
      .finally(() => setBookLoading(false));
  };

  const saveBook = (book: BookProject) => invoke<void>("save_book_project", { project: book })
    .then(() => {
      setActiveBook(book);
      setBookProjects((current) => current.map((item) => item.id === book.id ? book : item));
    });

  const translateBook = (book: BookProject, chapterId: string | null) => {
    if (!provider) {
      setProviderOpen(true);
      setBookError(`请先完成模型配置，再翻译${chapterId ? "本章" : "全书"}`);
      return;
    }
    setBookBusy(true);
    setBookError(null);
    void invoke<BookProject>("translate_book_project", {
      input: {
        runId: `book-${Date.now()}-${Math.random().toString(16).slice(2)}`,
        project: book,
        chapterId,
        provider: { ...provider, apiKey: null },
        sourceLanguage: languageFromCode(book.sourceLanguage, sourceLanguages),
        targetLanguage: languageFromCode(book.targetLanguage, targetLanguages),
      },
    })
      .then((translated) => {
        setActiveBook(translated);
        setBookProjects((current) => current.map((item) => item.id === translated.id ? translated : item));
      })
      .catch((error: unknown) => setBookError(applicationErrorMessage(error)))
      .finally(() => setBookBusy(false));
  };

  const exportBook = (book: BookProject, format: BookExportFormat, profile: BookExportProfile) => {
    setBookBusy(true);
    setBookError(null);
    void invoke<BookExportRecord>("export_book_project", { request: { project: book, format, profile } })
      .then((record) => setBookExportHistory((current) => [record, ...current.filter((item) => item.id !== record.id)]))
      .catch((error: unknown) => setBookError(applicationErrorMessage(error)))
      .finally(() => setBookBusy(false));
  };

  const openBookExport = (path: string) => {
    setBookError(null);
    void invoke("open_book_export_location", { path })
      .catch((error: unknown) => setBookError(applicationErrorMessage(error)));
  };

  const selectProject = () => {
    setOpenError(null);
    setScanning(true);
    void invoke<ProjectSummary>("select_and_scan_project")
      .then((result) => {
        setProject(result);
        savePreferences(result.projectPath);
        setHistory([]);
        setHistoryError(null);
        setHistoryNotice(null);
        setView("overview");
        setWorkspaceKind("game");
        window.location.hash = "game";
      })
      .catch((error: unknown) => setOpenError(String(error)))
      .finally(() => setScanning(false));
  };

  const changeBookStage = (stage: WorkspaceStage) => {
    if (stage === "projects") {
      closeBookWorkspace();
    } else if (stage === "history") {
      setView("history");
      if (activeBook) void loadBookExportHistory(activeBook.id);
    } else {
      setView(stage);
    }
  };

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

  const changeGameStage = (stage: WorkspaceStage) => {
    if (stage === "projects") {
      window.location.hash = "projects";
      setWorkspaceKind("library");
    } else if (stage === "history") {
      openHistory();
    } else {
      setView(stage);
    }
  };

  const closeGameProject = () => {
    setProject(null);
    savePreferences(null);
    setView("overview");
    setWorkspaceKind("library");
    window.location.hash = "projects";
  };

  if (workspaceKind === "library") {
    const libraryStage: WorkspaceStage = view === "history" ? "history" : "projects";
    return <>
      <WorkspaceChrome stage={libraryStage} projectType="studio" projectName="项目中心" providerName={provider?.model ?? null} projectOpen={false} onStageChange={(stage) => stage === "history" ? openHistory() : setView("overview")} onConfigure={() => setProviderOpen(true)}>
        {libraryStage === "history"
          ? <main className="content-stage"><PatchHistory entries={history} loading={historyLoading} error={historyError} notice={historyNotice} uninstallingId={uninstallingId} installingId={installingHistoryId} deletingId={deletingHistoryId} onInstall={installHistoryPatch} onUninstall={uninstallPatch} onDelete={deleteHistoryEntry} /></main>
          : <ProjectCenter projects={bookProjects} gameProject={project} loading={bookLoading} scanning={scanning} error={bookError ?? openError ?? persistenceError} onImportBook={importBook} onSelectGame={selectProject} onOpenBook={openBook} onOpenGame={() => { setView("overview"); setWorkspaceKind("game"); window.location.hash = "game"; }} />}
      </WorkspaceChrome>
      <ProviderDrawer open={providerOpen} current={provider} onClose={() => setProviderOpen(false)} onSave={saveProvider} />
    </>;
  }

  if (workspaceKind === "book" && activeBook) {
    return <>
      <WorkspaceChrome stage={view} projectType="book" projectName={activeBook.title} providerName={provider?.model ?? null} projectOpen status={bookBusy ? "任务进行中" : "自动保存"} onStageChange={changeBookStage} onConfigure={() => setProviderOpen(true)} onCloseProject={closeBookWorkspace}>
        <BookWorkspace project={activeBook} stage={view} providerName={provider?.model ?? null} busy={bookBusy} error={bookError} exportHistory={bookExportHistory} onSave={saveBook} onTranslate={translateBook} onExport={exportBook} onOpenExport={openBookExport} />
      </WorkspaceChrome>
      <ProviderDrawer open={providerOpen} current={provider} onClose={() => setProviderOpen(false)} onSave={saveProvider} />
    </>;
  }

  return (
    <>
      <WorkspaceChrome stage={view} projectType="game" projectName={project?.projectName ?? "游戏项目"} providerName={provider?.model ?? null} projectOpen={project !== null} status={translating ? "任务进行中" : project?.engine} onStageChange={changeGameStage} onConfigure={() => setProviderOpen(true)} onCloseProject={closeGameProject}>
        <main className="content-stage">
          {persistenceError ? <div className="notice-banner" role="alert"><span>ERROR</span>{persistenceError}</div> : null}
          {view === "overview" && project ? <LanguageSettings source={sourceLanguage} target={targetLanguage} onSourceChange={(language) => { setSourceLanguage(language); savePreferences(project.projectPath, language, targetLanguage); }} onTargetChange={(language) => { setTargetLanguage(language); savePreferences(project.projectPath, sourceLanguage, language); }} /> : null}
          {view === "overview" ? (
            <ProjectOverview
              project={project}
              configured={provider !== null}
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
      </WorkspaceChrome>
      <ProviderDrawer
        open={providerOpen}
        current={provider}
        onClose={() => setProviderOpen(false)}
        onSave={saveProvider}
      />
    </>
  );
}

function currentTime() {
  return new Date().toLocaleTimeString("zh-CN", { hour12: false });
}

function languageFromCode(code: string, options: Language[]): Language {
  return options.find((language) => language.code === code) ?? { code, name: code };
}
