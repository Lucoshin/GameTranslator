export type Language = { code: string; name: string };

export const sourceLanguages: Language[] = [
  { code: "auto", name: "自动检测" },
  { code: "zh-CN", name: "简体中文" }, { code: "zh-TW", name: "繁体中文" },
  { code: "en-US", name: "英语" }, { code: "ja-JP", name: "日语" },
  { code: "ko-KR", name: "韩语" }, { code: "fr-FR", name: "法语" },
  { code: "de-DE", name: "德语" }, { code: "es-ES", name: "西班牙语" },
  { code: "ru-RU", name: "俄语" }, { code: "pt-BR", name: "葡萄牙语" },
];

export const targetLanguages = sourceLanguages.slice(1);

export function LanguageSettings({
  source,
  target,
  onSourceChange,
  onTargetChange,
}: {
  source: Language;
  target: Language;
  onSourceChange: (language: Language) => void;
  onTargetChange: (language: Language) => void;
}) {
  return (
    <div className="language-settings" aria-label="翻译语言">
      <LanguageSelect label="源语言" value={source} options={sourceLanguages} onChange={onSourceChange} />
      <span className="language-arrow">→</span>
      <LanguageSelect label="目标语言" value={target} options={targetLanguages} onChange={onTargetChange} />
    </div>
  );
}

function LanguageSelect({ label, value, options, onChange }: {
  label: string;
  value: Language;
  options: Language[];
  onChange: (language: Language) => void;
}) {
  const custom = !options.some((language) => language.code === value.code);
  return (
    <div className="language-field">
      <label>{label}
        <select aria-label={label} value={custom ? "custom" : value.code} onChange={(event) => {
          if (event.target.value === "custom") {
            onChange({ code: "x-custom", name: "自定义语言" });
          } else {
            onChange(options.find((language) => language.code === event.target.value)!);
          }
        }}>
          {options.map((language) => <option key={language.code} value={language.code}>{language.name} · {language.code}</option>)}
          <option value="custom">自定义语言…</option>
        </select>
      </label>
      {custom ? (
        <div className="custom-language">
          <input aria-label={`${label}名称`} value={value.name} onChange={(event) => onChange({ ...value, name: event.target.value })} placeholder="语言名称" />
          <input aria-label={`${label}代码`} value={value.code} onChange={(event) => onChange({ ...value, code: event.target.value })} placeholder="BCP 47 代码" />
        </div>
      ) : null}
    </div>
  );
}
