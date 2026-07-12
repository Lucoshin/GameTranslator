import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ExportPanel } from "./features/export/ExportPanel";
import { ProjectHome } from "./features/projects/ProjectHome";
import { ProjectOverview, type ProjectSummary } from "./features/projects/ProjectOverview";
import { ProviderDrawer, type ProviderConfiguration } from "./features/providers/ProviderDrawer";
import { SegmentTable } from "./features/review/SegmentTable";
import { TranslationProgress } from "./features/translation/TranslationProgress";
import { LanguageSettings, type Language } from "./features/translation/LanguageSettings";
import "./styles/global.css";

type View = "overview" | "translation" | "review" | "export";
export type TranslationItem = { id: string; source: string; target: string; speaker: string | null; sourceFile: string; qa: "passed" | "warning" | "blocking" };
export type TranslationRun = { items: TranslationItem[]; warningFindings: number; blockingFindings: number; failedSegmentIds: string[] };
export type TranslationProgressState = { phase: "idle" | "extracting" | "translating" | "qa" | "completed" | "failed"; completed: number; total: number; failed: number; warningFindings: number; blockingFindings: number; message: string; concurrency?: number; throughput?: number; etaSeconds?: number };
export type TranslationLog = { time: string; message: string };
type TranslationProgressEvent = TranslationProgressState & { runId: string };
type ExportResult = { outputPath: string; fileCount: number };

export default function App() {
  const [project, setProject] = useState<ProjectSummary | null>(null);
  const [openError, setOpenError] = useState<string | null>(null);
  const [scanning, setScanning] = useState(false);
  const [view, setView] = useState<View>("overview");
  const [providerOpen, setProviderOpen] = useState(false);
  const [provider, setProvider] = useState<ProviderConfiguration | null>(() => {
    const stored = localStorage.getItem("game-translator-provider");
    return stored ? JSON.parse(stored) as ProviderConfiguration : null;
  });
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

  const saveProvider = (configuration: ProviderConfiguration) => {
    void invoke("save_provider_configuration", { provider: configuration })
      .then(() => {
        const persisted = { ...configuration, apiKey: undefined };
        localStorage.setItem("game-translator-provider", JSON.stringify(persisted));
        setProvider(persisted);
        setProviderOpen(false);
      })
      .catch((error: unknown) => setTranslationError(String(error)));
  };

  if (!project) {
    return <>
      <ProjectHome
        error={openError}
        scanning={scanning}
        providerName={provider?.model ?? null}
        onConfigure={() => setProviderOpen(true)}
        onSelect={() => {
          setOpenError(null);
          setScanning(true);
          void invoke<Omit<ProjectSummary, "demo">>("select_and_scan_project")
            .then((result) => setProject({ ...result, demo: false }))
            .catch((error: unknown) => setOpenError(String(error)))
            .finally(() => setScanning(false));
        }}
        onOpenDemo={() => setProject({
          projectPath: "D:\\Games\\Moonlit Shrine",
          projectName: "月影神殿",
          engine: "RPG Maker MZ",
          segmentCount: 1284,
          demo: true,
        })}
      />
      <ProviderDrawer open={providerOpen} current={provider} onClose={() => setProviderOpen(false)} onSave={saveProvider} />
    </>;
  }

  const startTranslation = () => {
    if (project.demo) {
      setView("translation");
      return;
    }
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
    if (project.demo) return;
    if (!translation) {
      setExportError("没有可导出的翻译结果");
      return;
    }
    setExportError(null);
    setExporting(true);
    void invoke<ExportResult>("export_translation_patch", {
      input: { projectPath: project.projectPath, items: translation.items, targetLanguage },
    })
      .then(setExportResult)
      .catch((error: unknown) => setExportError(String(error)))
      .finally(() => setExporting(false));
  };

  return (
    <div className="app-shell">
      <aside className="rail">
        <button className="brand-mark" onClick={() => setView("overview")} aria-label="返回项目概览">
          <span>译</span>
        </button>
        <nav aria-label="项目导航">
          <RailButton label="概览" active={view === "overview"} onClick={() => setView("overview")} glyph="本" />
          <RailButton label="任务" active={view === "translation"} onClick={() => setView("translation")} glyph="进" />
          <RailButton label="校对" active={view === "review"} onClick={() => setView("review")} glyph="校" />
          <RailButton label="导出" active={view === "export"} onClick={() => setView("export")} glyph="出" />
        </nav>
        <button className="rail-exit" onClick={() => setProject(null)} aria-label="关闭项目">
          ×
        </button>
      </aside>

      <div className="workspace">
        <header className="topbar">
          <div>
            <span className="eyebrow">PROJECT / {project.demo ? "演示项目" : "本地项目"}</span>
            <strong>{project.projectName}</strong>
          </div>
          <div className="topbar-actions">
            <span className="model-chip"><i />{provider?.model ?? "未配置"}</span>
            <button className="text-button" aria-label="顶部配置模型" onClick={() => setProviderOpen(true)}>配置模型</button>
          </div>
        </header>

        <main className="content-stage">
          {view === "overview" ? <LanguageSettings source={sourceLanguage} target={targetLanguage} onSourceChange={setSourceLanguage} onTargetChange={setTargetLanguage} /> : null}
          {view === "overview" ? (
            <ProjectOverview
              project={project}
              configured={provider !== null}
              onConfigure={() => setProviderOpen(true)}
              onStart={startTranslation}
            />
          ) : null}
          {view === "translation" ? (
            <TranslationProgress
              result={project.demo ? null : translation}
              demo={project.demo}
              loading={translating}
              error={translationError}
              progress={progress}
              logs={translationLogs}
              onReview={() => setView("review")}
              onExport={() => setView("export")}
            />
          ) : null}
          {view === "review" ? (
            <SegmentTable
              items={project.demo ? undefined : translation?.items}
              onChange={(id, target) => setTranslation((current) => current ? {
                ...current,
                items: current.items.map((item) => item.id === id ? { ...item, target } : item),
              } : current)}
              onExport={() => setView("export")}
              targetLanguage={targetLanguage}
              sourceLanguage={sourceLanguage}
            />
          ) : null}
          {view === "export" ? (
            <ExportPanel
              demo={project.demo}
              result={exportResult}
              error={exportError}
              exporting={exporting}
              canExport={project.demo || translation !== null}
              onExport={exportPatch}
              targetLanguage={targetLanguage}
            />
          ) : null}
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
  glyph,
  active,
  onClick,
}: {
  label: string;
  glyph: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button className={active ? "rail-button active" : "rail-button"} onClick={onClick} aria-label={label}>
      <span>{glyph}</span>
      <small>{label}</small>
    </button>
  );
}
