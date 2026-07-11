import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ExportPanel } from "./features/export/ExportPanel";
import { ProjectHome } from "./features/projects/ProjectHome";
import { ProjectOverview, type ProjectSummary } from "./features/projects/ProjectOverview";
import { ProviderDrawer, type ProviderConfiguration } from "./features/providers/ProviderDrawer";
import { SegmentTable } from "./features/review/SegmentTable";
import { TranslationProgress } from "./features/translation/TranslationProgress";
import "./styles/global.css";

type View = "overview" | "translation" | "review" | "export";
export type TranslationItem = { id: string; source: string; target: string; speaker: string | null; sourceFile: string };
export type TranslationRun = { items: TranslationItem[]; warningFindings: number; blockingFindings: number; failedSegmentIds: string[] };
type ExportResult = { outputPath: string; fileCount: number };

export default function App() {
  const [project, setProject] = useState<ProjectSummary | null>(null);
  const [openError, setOpenError] = useState<string | null>(null);
  const [view, setView] = useState<View>("overview");
  const [providerOpen, setProviderOpen] = useState(false);
  const [provider, setProvider] = useState<ProviderConfiguration | null>(null);
  const [translation, setTranslation] = useState<TranslationRun | null>(null);
  const [translationError, setTranslationError] = useState<string | null>(null);
  const [translating, setTranslating] = useState(false);
  const [exportResult, setExportResult] = useState<ExportResult | null>(null);
  const [exportError, setExportError] = useState<string | null>(null);
  const [exporting, setExporting] = useState(false);

  if (!project) {
    return (
      <ProjectHome
        error={openError}
        onSelect={() => {
          setOpenError(null);
          void invoke<Omit<ProjectSummary, "demo">>("select_and_scan_project")
            .then((result) => setProject({ ...result, demo: false }))
            .catch((error: unknown) => setOpenError(String(error)));
        }}
        onOpenDemo={() => setProject({
          projectPath: "D:\\Games\\Moonlit Shrine",
          projectName: "月影神殿",
          engine: "RPG Maker MZ",
          segmentCount: 1284,
          demo: true,
        })}
      />
    );
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
    setTranslating(true);
    setView("translation");
    void invoke<TranslationRun>("translate_project", {
      input: {
        projectPath: project.projectPath,
        provider: { ...provider, apiKey: null },
      },
    })
      .then(setTranslation)
      .catch((error: unknown) => setTranslationError(String(error)))
      .finally(() => setTranslating(false));
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
      input: { projectPath: project.projectPath, items: translation.items },
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
            />
          ) : null}
        </main>
      </div>

      <ProviderDrawer
        open={providerOpen}
        current={provider}
        onClose={() => setProviderOpen(false)}
        onSave={(configuration) => {
          void invoke("save_provider_configuration", { provider: configuration })
            .then(() => {
              setProvider(configuration);
              setProviderOpen(false);
            })
            .catch((error: unknown) => setTranslationError(String(error)));
        }}
      />
    </div>
  );
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
