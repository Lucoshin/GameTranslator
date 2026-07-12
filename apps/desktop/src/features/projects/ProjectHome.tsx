export function ProjectHome({ onSelect, onEnterOverview, onConfigure, onReturn, providerName, error, scanning }: { onSelect: () => void; onEnterOverview: () => void; onConfigure: () => void; onReturn?: () => void; providerName: string | null; error: string | null; scanning: boolean }) {
  return (
    <main className="home-screen">
      <div className="paper-grain" />
      <header className="home-header">
        {onReturn ? <button className="wordmark home-return" aria-label="返回项目概览" onClick={onReturn}><span className="seal">译</span><span>GameTranslator</span></button> : <button className="wordmark" aria-label="进入项目概览" onClick={onEnterOverview}><span className="seal">译</span><span>GameTranslator</span></button>}
        <div className="home-settings"><span className={providerName ? "model-chip" : "model-chip inactive"}><i />{providerName ?? "模型未配置"}</span><button className="text-button" aria-label="主界面配置模型" onClick={onConfigure}>配置模型</button><span className="version">OPEN SOURCE · v0.1 PREVIEW</span></div>
      </header>

      <section className="hero" id="top">
        <div className="hero-copy">
          <p className="kicker">游戏文本，本地处理</p>
          <h1>让另一种语言，<br /><em>住进同一个世界。</em></h1>
          <p className="hero-lead">
            支持 RPG Maker、Ren'Py 与 RimWorld 模组的本地化工作台。你持有模型密钥，原始内容保持不变。
          </p>
          <div className="hero-actions">
            <button className="primary-action" aria-label="选择内容目录" disabled={scanning} onClick={onSelect}>
              {scanning ? "正在识别并提取文本…" : "选择游戏或模组目录"} <span>↗</span>
            </button>
          </div>
          {error ? <p className="home-error" role="alert">{error}</p> : null}
          <p className="availability">支持 RPG Maker MV / MZ、Ren'Py 8.x 与 RimWorld 英文语言包模组</p>
        </div>

        <div className="hero-art" aria-hidden="true">
          <div className="moon-disc" />
          <div className="script-sheet sheet-back">
            <span>MAP 001 / EVENT 04</span>
            <i />
            <i />
            <i className="short" />
          </div>
          <div className="script-sheet sheet-front">
            <span className="sheet-label">TRANSLATION</span>
            <div className="proof-mark">译</div>
          </div>
          <div className="vertical-type">物語を、もっと近くへ</div>
        </div>
      </section>

      <footer className="home-footer">
        <span>01 自动识别</span><span>02 语境翻译</span><span>03 安全补丁</span>
      </footer>
    </main>
  );
}
