import type { ReactNode } from "react";
import "./workspace-chrome.css";

export type WorkspaceStage = "projects" | "overview" | "translation" | "review" | "export" | "history";

export function WorkspaceChrome({
  stage,
  projectType,
  projectName,
  providerName,
  projectOpen,
  status,
  onStageChange,
  onConfigure,
  onCloseProject,
  children,
}: {
  stage: WorkspaceStage;
  projectType: "studio" | "game" | "book";
  projectName: string;
  providerName: string | null;
  projectOpen: boolean;
  status?: string;
  onStageChange: (stage: WorkspaceStage) => void;
  onConfigure: () => void;
  onCloseProject?: () => void;
  children: ReactNode;
}) {
  const typeLabel = projectType === "game" ? "GAME / 游戏项目" : projectType === "book" ? "BOOK / 书籍项目" : "STUDIO / 项目工作室";
  return <div className={`app-shell unified-shell ${projectType}`}>
    <aside className="rail unified-rail">
      <button className="brand-mark" onClick={() => onStageChange("projects")} aria-label="返回项目中心"><span>译</span></button>
      <nav aria-label="应用导航">
        <WorkspaceNavButton label="项目" stage="projects" active={stage === "projects"} onClick={onStageChange} icon="projects" />
        <WorkspaceNavButton label="概览" stage="overview" active={stage === "overview"} disabled={!projectOpen} onClick={onStageChange} icon="overview" />
        <WorkspaceNavButton label="翻译" stage="translation" active={stage === "translation"} disabled={!projectOpen} onClick={onStageChange} icon="translation" />
        <WorkspaceNavButton label="校对" stage="review" active={stage === "review"} disabled={!projectOpen} onClick={onStageChange} icon="review" />
        <WorkspaceNavButton label="导出" stage="export" active={stage === "export"} disabled={!projectOpen} onClick={onStageChange} icon="export" />
        <WorkspaceNavButton label="历史" stage="history" active={stage === "history"} onClick={onStageChange} icon="history" />
      </nav>
      {projectOpen && onCloseProject ? <button className="rail-exit" onClick={onCloseProject} aria-label="关闭项目">×</button> : <span className="unified-rail-version">GT</span>}
    </aside>
    <div className="workspace unified-workspace">
      <header className="topbar unified-topbar">
        <div className="unified-project-title">
          <span className="eyebrow">{typeLabel}</span>
          {projectType === "game" ? <strong className="unified-project-name">{projectName}</strong> : <h1>{projectName}</h1>}
        </div>
        <div className="topbar-actions">
          {status ? <span className="unified-status">{status}</span> : null}
          <button className={providerName ? "model-chip" : "model-chip inactive"} onClick={onConfigure} aria-label="配置模型"><i />{providerName ?? "模型未配置"}</button>
        </div>
      </header>
      {children}
    </div>
  </div>;
}

function WorkspaceNavButton({ label, stage, active, disabled = false, onClick, icon }: {
  label: string;
  stage: WorkspaceStage;
  active: boolean;
  disabled?: boolean;
  onClick: (stage: WorkspaceStage) => void;
  icon: "projects" | "overview" | "translation" | "review" | "export" | "history";
}) {
  return <button className={active ? "rail-button active" : "rail-button"} disabled={disabled} onClick={() => onClick(stage)} aria-label={label}>
    <WorkspaceIcon icon={icon} /><small>{label}</small>
  </button>;
}

function WorkspaceIcon({ icon }: { icon: "projects" | "overview" | "translation" | "review" | "export" | "history" }) {
  const path = icon === "projects" ? <><path d="M4 5h16v14H4z" /><path d="M4 9h16M9 9v10" /></>
    : icon === "overview" ? <><rect x="3" y="3" width="7" height="7" /><rect x="14" y="3" width="7" height="7" /><rect x="3" y="14" width="7" height="7" /><rect x="14" y="14" width="7" height="7" /></>
      : icon === "translation" ? <><circle cx="12" cy="12" r="8.5" /><path d="m10 8 6 4-6 4z" /></>
        : icon === "review" ? <><path d="m4 17.5.7-4.2L15.8 2.2l4 4L8.7 17.3z" /><path d="m13.4 4.6 4 4" /><path d="M4 21h16" /></>
          : icon === "export" ? <><path d="M12 3v11" /><path d="m8 10 4 4 4-4" /><path d="M4 15v5h16v-5" /></>
            : <><circle cx="12" cy="12" r="8.5" /><path d="M12 7v5l3.5 2" /></>;
  return <svg className="rail-icon" viewBox="0 0 24 24" aria-hidden="true">{path}</svg>;
}
