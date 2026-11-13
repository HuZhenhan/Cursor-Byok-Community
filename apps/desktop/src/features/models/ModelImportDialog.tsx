import type { ModelType } from "../../shared/api";
import { ConfirmDialog } from "../../shared/ui/ConfirmDialog";
import styles from "./ModelImportDialog.module.scss";

export type ModelImportDialogItem = {
  key: string;
  display_name: string;
  model_id: string;
  type: ModelType;
  existing: boolean;
};

/** 旧版配置与 JSON 文件导入共用的确认对话框,展示来源和逐个模型的新增/已存在状态。 */
export function ModelImportDialog({
  open,
  title,
  source,
  items,
  total,
  newModels,
  existingModels,
  hint,
  busy,
  onCancel,
  onConfirm,
}: {
  open: boolean;
  title: string;
  source: string;
  items: ModelImportDialogItem[];
  total: number;
  newModels: number;
  existingModels: number;
  hint: string;
  busy: boolean;
  onCancel: () => void;
  onConfirm: () => void;
}) {
  return <ConfirmDialog
    open={open}
    title={title}
    busy={busy}
    wide
    cancelLabel={t("取消")}
    confirmLabel={t("确认导入")}
    onCancel={onCancel}
    onConfirm={onConfirm}
  >
    <div className={styles.content}>
      <div className={styles.source}>
        <strong>{t("配置文件")}</strong>
        <code>{source}</code>
      </div>
      <div className={styles.counts}>
        <div><strong>{total}</strong><small>{t("配置模型")}</small></div>
        <div><strong>{newModels}</strong><small>{t("将新增")}</small></div>
        <div><strong>{existingModels}</strong><small>{t("已存在")}</small></div>
      </div>
      <div className={styles.models}>
        {items.map((model) => <div className={styles.model} key={model.key}>
          <div>
            <strong>{model.display_name}</strong>
            <small>{model.model_id} · {model.type === "openai" ? "OpenAI" : "Anthropic"}</small>
          </div>
          <span className={model.existing ? styles.existing : styles.new}>
            {model.existing ? t("已存在，跳过") : t("新增")}
          </span>
        </div>)}
      </div>
      <small className={styles.hint}>{hint}</small>
    </div>
  </ConfirmDialog>;
}
