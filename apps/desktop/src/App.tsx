import { useState } from "react";
import { ExportPanel } from "./features/export/ExportPanel";
import { ProjectHome } from "./features/projects/ProjectHome";
import { ProjectOverview } from "./features/projects/ProjectOverview";
import { ProviderDrawer } from "./features/providers/ProviderDrawer";
import { SegmentTable } from "./features/review/SegmentTable";
import { TranslationProgress } from "./features/translation/TranslationProgress";
import "./styles/global.css";

type View = "overview" | "translation" | "review" | "export";

export default function App() {
  const [projectOpen, setProjectOpen] = useState(false);
  const [view, setView] = useState<View>("overview");
  const [providerOpen, setProviderOpen] = useState(false);
  const [model, setModel] = useState("未配置");

  if (!projectOpen) {
    return <ProjectHome onOpenDemo={() => setProjectOpen(true)} />;
  }

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
        <button className="rail-exit" onClick={() => setProjectOpen(false)} aria-label="关闭项目">
          ×
        </button>
      </aside>

      <div className="workspace">
        <header className="topbar">
          <div>
            <span className="eyebrow">PROJECT / 演示项目</span>
            <strong>月影神殿</strong>
          </div>
          <div className="topbar-actions">
            <span className="model-chip"><i />{model}</span>
            <button className="text-button" aria-label="顶部配置模型" onClick={() => setProviderOpen(true)}>配置模型</button>
          </div>
        </header>

        <main className="content-stage">
          {view === "overview" ? (
            <ProjectOverview
              onConfigure={() => setProviderOpen(true)}
              onStart={() => setView("translation")}
            />
          ) : null}
          {view === "translation" ? (
            <TranslationProgress
              onReview={() => setView("review")}
              onExport={() => setView("export")}
            />
          ) : null}
          {view === "review" ? <SegmentTable onExport={() => setView("export")} /> : null}
          {view === "export" ? <ExportPanel /> : null}
        </main>
      </div>

      <ProviderDrawer
        open={providerOpen}
        currentModel={model === "未配置" ? "" : model}
        onClose={() => setProviderOpen(false)}
        onSave={(nextModel) => {
          setModel(nextModel);
          setProviderOpen(false);
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
