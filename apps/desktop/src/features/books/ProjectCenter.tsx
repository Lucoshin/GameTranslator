import { bookIssueCount, bookProgress, type BookProject } from "./contracts";
import type { ProjectSummary } from "../projects/ProjectOverview";
import "./book-workspace.css";

export function ProjectCenter({ projects, gameProject, loading, scanning, error, onImportBook, onSelectGame, onOpenBook, onOpenGame }: {
  projects: BookProject[];
  gameProject: ProjectSummary | null;
  loading: boolean;
  scanning: boolean;
  error: string | null;
  onImportBook: () => void;
  onSelectGame: () => void;
  onOpenBook: (project: BookProject) => void;
  onOpenGame: () => void;
}) {
  return <main className="library-center unified-project-center">
    <header className="library-header">
      <div><p>PROJECT LIBRARY / 项目中心</p><h1>所有项目</h1><span>游戏本地化与书籍译校，共用同一条工作流。</span></div>
      <div className="library-header-actions">
        <button type="button" className="library-game-import" aria-label="选择内容目录" onClick={onSelectGame} disabled={scanning}>＋ {scanning ? "正在识别…" : "选择游戏目录"}</button>
        <button type="button" className="library-create" onClick={onImportBook} disabled={loading}>＋ {loading ? "正在导入…" : "导入书籍"}</button>
      </div>
    </header>
    {error ? <div className="library-error" role="alert">{error}</div> : null}
    <section className="unified-import-guide" aria-label="统一工作流程">
      <span>01 导入内容</span><i />
      <span>02 语境翻译</span><i />
      <span>03 人工校对</span><i />
      <span>04 独立导出</span>
    </section>
    {!loading && projects.length === 0 && !gameProject ? <section className="library-empty" aria-label="尚无书籍项目"><strong>从一种内容开始</strong><p>游戏目录与书籍文件都会进入同一项目工作室；原始内容始终保持不变。</p><div><button type="button" aria-label="空状态选择游戏目录" onClick={onSelectGame}>选择游戏目录</button><button type="button" onClick={onImportBook}>选择书籍文件</button></div></section> : null}
    <section aria-label="项目列表" className="library-grid">
      {projects.map((project) => {
        const progress = bookProgress(project);
        const issues = bookIssueCount(project);
        return <article className="library-card book" key={project.id}>
          <div className="library-book-cover"><span>{project.title.slice(0, 4)}</span><i>{project.format.toUpperCase()}</i></div>
          <div className="library-card-copy">
            <span className="library-type">书籍 · {project.format.toUpperCase()}</span>
            <h2>《{project.title}》</h2><p>{project.sourcePath}</p>
            <div className="library-progress"><span><i style={{ width: `${progress}%` }} /></span><b>{progress}%</b></div>
            <p className="library-meta">{project.chapters.length} 章 · {issues} 个待处理问题</p>
            <button type="button" className="library-card-action" aria-label={`打开《${project.title}》`} onClick={() => onOpenBook(project)}>继续译校 <span>→</span></button>
          </div>
        </article>;
      })}
      {gameProject ? <article className="library-card game recent-game">
        <div className="library-game-art"><span>游</span><i /></div>
        <div className="library-card-copy">
          <span className="library-type">游戏 · 最近项目</span>
          <h2>{gameProject.projectName}</h2><p>{gameProject.projectPath}</p>
          <div className="library-progress game-progress"><span><i /></span><b>READY</b></div>
          <p className="library-meta">{gameProject.engine} · {gameProject.segmentCount} 条可翻译文本</p>
          <button type="button" className="library-card-action" aria-label={`继续游戏项目 ${gameProject.projectName}`} onClick={onOpenGame}>继续工作 <span>→</span></button>
        </div>
      </article> : <article className="library-card game">
        <div className="library-game-art"><span>游</span><i /></div>
        <div className="library-card-copy">
          <span className="library-type">游戏</span>
          <h2>导入游戏或模组</h2><p>RPG Maker · Ren'Py · RimWorld</p>
          <div className="library-progress game-progress"><span><i /></span><b>READY</b></div>
          <p className="library-meta">扫描目录后进入统一的概览、翻译、校对与导出流程</p>
          <button type="button" className="library-card-action secondary" aria-label="进入游戏工作台" onClick={onSelectGame}>选择游戏目录 <span>→</span></button>
        </div>
      </article>}
    </section>
  </main>;
}
