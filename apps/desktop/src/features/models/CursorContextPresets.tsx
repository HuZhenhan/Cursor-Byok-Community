import styles from "./CursorSettings.module.scss";

/** 上下文窗口档位,与服务端 Cursor 上下文分档保持一致。 */
const CONTEXT_PRESETS = [
  { label: "128K", tokens: 128000 },
  { label: "200K", tokens: 200000 },
  { label: "356K", tokens: 356000 },
  { label: "500K", tokens: 500000 },
  { label: "800K", tokens: 800000 },
  { label: "1M", tokens: 1000000 },
];

/** 上下文窗口档位选择器:点击档位填入对应 Token 数,再次点击清空为默认值。 */
export function CursorContextPresets({ value, onChange }: {
  value: number | null;
  onChange: (tokens: number | null) => void;
}) {
  return <div className={styles.contextPresets} role="group" aria-label={t("上下文窗口档位")}>
    {CONTEXT_PRESETS.map((preset) => <button
      key={preset.tokens}
      type="button"
      aria-pressed={value === preset.tokens}
      onClick={() => onChange(value === preset.tokens ? null : preset.tokens)}
    >
      {preset.label}
    </button>)}
  </div>;
}
