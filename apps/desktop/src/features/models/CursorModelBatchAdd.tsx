import { useState } from "react";
import { Checkbox } from "../../shared/ui/Checkbox";
import controls from "../../shared/ui/Controls.module.scss";
import styles from "./CursorSettings.module.scss";

/** 获取模型之后的添加列表:逐项勾选后批量添加,也可一次性添加整组。 */
export function CursorModelBatchAdd({ models, existingModelIds, busy, onAdd }: {
  models: string[];
  existingModelIds: string[];
  busy: boolean;
  onAdd: (modelIds: string[]) => void;
}) {
  const [selected, setSelected] = useState<ReadonlySet<string>>(() => new Set());
  const available = models.filter((modelId) => !existingModelIds.includes(modelId));

  const toggle = (modelId: string, checked: boolean) => {
    setSelected((current) => {
      const next = new Set(current);
      if (checked) next.add(modelId);
      else next.delete(modelId);
      return next;
    });
  };

  return <div className={styles.batchAdd}>
    <div className={styles.batchAddHeader}>
      <span className={styles.batchAddTitle}>{t("获取到的模型（{count}）", { count: models.length })}</span>
      <button type="button" className={controls.secondary} disabled={busy || available.length === 0} onClick={() => onAdd(available)}>{available.length > 0 ? t("全部添加") : t("已全部添加")}</button>
      <button type="button" className={controls.primary} disabled={busy || selected.size === 0} onClick={() => onAdd([...selected])}>
        {busy ? t("添加中…") : t("批量添加（{count}）", { count: selected.size })}
      </button>
    </div>
    <div className={styles.batchAddList}>
      {models.map((modelId) => <Checkbox
        key={modelId}
        checked={selected.has(modelId)}
        disabled={busy || existingModelIds.includes(modelId)}
        label={existingModelIds.includes(modelId) ? `${modelId} ${t("（已添加）")}` : modelId}
        onChange={(checked) => toggle(modelId, checked)}
      />)}
    </div>
  </div>;
}
