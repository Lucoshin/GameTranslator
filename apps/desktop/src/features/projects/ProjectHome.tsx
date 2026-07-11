export function ProjectHome({ onOpenDemo }: { onOpenDemo: () => void }) {
  return (
    <main className="home-screen">
      <div className="paper-grain" />
      <header className="home-header">
        <a className="wordmark" href="#top" aria-label="GameTranslator 首页">
          <span className="seal">译</span>
          <span>GameTranslator</span>
        </a>
        <span className="version">OPEN SOURCE · v0.1 PREVIEW</span>
      </header>

      <section className="hero" id="top">
        <div className="hero-copy">
          <p className="kicker">游戏文本，本地处理</p>
          <h1>让另一种语言，<br /><em>住进同一个世界。</em></h1>
          <p className="hero-lead">
            为 RPG Maker MV / MZ 打造的一键汉化工作台。你持有模型密钥，原始游戏保持不变。
          </p>
          <div className="hero-actions">
            <button className="primary-action" disabled title="目录选择将在桌面命令接入后启用">
              选择游戏目录 <span>↗</span>
            </button>
            <button className="ghost-action" onClick={onOpenDemo}>载入演示项目</button>
          </div>
          <p className="availability">首版支持未加密 RPG Maker MV / MZ 项目</p>
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
            <p>「やっと着いた。」</p>
            <div className="proof-mark">译</div>
            <p className="translated">“终于到了。”</p>
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

