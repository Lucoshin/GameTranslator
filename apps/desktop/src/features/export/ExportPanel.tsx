export function ExportPanel() {
  return (
    <div className="page export-page">
      <section className="page-heading compact"><div><p className="kicker">PATCH BUILDER</p><h1>导出汉化补丁</h1><p className="muted">原游戏文件不会被直接修改</p></div><span className="stamp-status small">校验通过</span></section>
      <div className="export-layout">
        <section className="panel export-summary">
          <div className="panel-title"><span>导出清单</span><small>PATCH MANIFEST</small></div>
          <div className="file-stack"><i/><i/><i/><div><strong>24</strong><span>个翻译文件</span></div></div>
          <dl><div><dt>补丁格式</dt><dd>GameTranslator Patch v1</dd></div><div><dt>目标语言</dt><dd>简体中文 zh-CN</dd></div><div><dt>完整性</dt><dd className="safe">SHA-256 已校验</dd></div></dl>
        </section>
        <section className="panel export-action">
          <div className="safety-callout"><span>只读</span><div><b>原始游戏保持不变</b><p>补丁将写入新的目录，并附带文件哈希与恢复说明。</p></div></div>
          <label>导出位置<div className="path-field"><span>D:\GameTranslator\Exports\MoonlitShrine-zhCN</span><button>选择</button></div></label>
          <label className="check-option"><input type="checkbox" defaultChecked /> 导出后打开文件夹</label>
          <button className="primary-action full">生成汉化补丁 <span>↗</span></button>
        </section>
      </div>
    </div>
  );
}

