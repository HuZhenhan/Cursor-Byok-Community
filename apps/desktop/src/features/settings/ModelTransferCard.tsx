import { useState } from "react";
import { api, type ModelImportPreview, type ModelInput } from "../../shared/api";
import { appStore, useAppStore } from "../../shared/store/appStore";
import { Button } from "../../shared/ui/Button";
import { TitledCard } from "../../shared/ui/TitledCard";
import { useMessage } from "../../shared/ui/message";
import { LegacyModelImport } from "../models/LegacyModelImport";
import { ModelImportDialog } from "../models/ModelImportDialog";
import styles from "./ModelTransferCard.module.scss";

type PendingImport = {
  path: string;
  models: ModelInput[];
  preview: ModelImportPreview;
};

export function ModelTransferCard() {
  const { models } = useAppStore();
  const message = useMessage();
  const [exporting, setExporting] = useState(false);
  const [reading, setReading] = useState(false);
  const [importing, setImporting] = useState(false);
  const [pending, setPending] = useState<PendingImport | null>(null);

  const exportModels = async () => {
    try {
      setExporting(true);
      const path = await api.saveModelExport();
      if (path) message(t("已导出到 {path}", { path }));
    } catch (cause) {
      message(errorText(cause));
    } finally {
      setExporting(false);
    }
  };

  const chooseImportFile = async () => {
    try {
      setReading(true);
      const file = await api.openModelImportFile();
      if (!file) return;
      const inputs = parseModelFile(file.contents);
      setPending({ path: file.path, models: inputs, preview: await api.previewModelImport(inputs) });
    } catch (cause) {
      message(errorText(cause));
    } finally {
      setReading(false);
    }
  };

  const confirmImport = async () => {
    if (!pending) return;
    setImporting(true);
    try {
      const result = await appStore.importModels(pending.models);
      if (!result) {
        const error = appStore.getSnapshot().error;
        if (error) message(error);
        return;
      }
      setPending(null);
      message(result.imported > 0
        ? t("导入完成：新增 {imported} 个模型，跳过 {skipped} 个已存在模型", { imported: result.imported, skipped: result.skipped })
        : t("配置中的 {count} 个模型均已存在，无需重复导入", { count: result.skipped }));
    } finally {
      setImporting(false);
    }
  };

  return <>
    <LegacyModelImport>{({ busy, previewing, open }) => <TitledCard title={t("模型导入导出")}>
      <div className={styles.row}>
        <div>
          <strong>{t("导出到文件")}</strong>
          <small>{t("把全部模型配置写入 JSON 文件，包含 API Key、扩展参数和自定义请求头；文件为明文，请妥善保管。")}</small>
        </div>
        <Button size="small" disabled={exporting || models.length === 0} onClick={() => void exportModels()}>{exporting ? t("导出中…") : t("导出模型")}</Button>
      </div>
      <div className={styles.row}>
        <div>
          <strong>{t("从文件导入")}</strong>
          <small>{t("从导出的 JSON 文件恢复模型配置；确认前会显示新增和已存在的模型。")}</small>
        </div>
        <Button size="small" disabled={reading || importing || busy} onClick={() => void chooseImportFile()}>{reading ? t("读取中…") : t("导入模型")}</Button>
      </div>
      <div className={styles.row}>
        <div>
          <strong>{t("旧版配置")}</strong>
          <small>{t("从本机旧版配置读取模型；确认前会显示新增和已存在的模型。")}</small>
        </div>
        <Button size="small" disabled={busy} onClick={open}>{previewing ? t("读取中…") : t("查看并导入")}</Button>
      </div>
    </TitledCard>}</LegacyModelImport>
    <ModelImportDialog
      open={pending !== null}
      title={t("导入模型")}
      source={pending?.path ?? ""}
      items={(pending?.preview.models ?? []).map((model) => ({
        key: model.model_hash,
        display_name: model.display_name,
        model_id: model.model_id,
        type: model.type,
        existing: model.existing,
      }))}
      total={pending?.preview.total ?? 0}
      newModels={pending?.preview.new_models ?? 0}
      existingModels={pending?.preview.existing_models ?? 0}
      hint={t("导入的模型会追加到列表末尾；文件中已存在的模型会自动跳过。")}
      busy={importing}
      onCancel={() => setPending(null)}
      onConfirm={() => void confirmImport()}
    />
  </>;
}

/** 读取导出文件的 `{ models: [...] }` 结构;单个模型的字段校验交给服务端。 */
function parseModelFile(contents: string): ModelInput[] {
  let parsed: unknown;
  try {
    parsed = JSON.parse(contents);
  } catch {
    throw new Error(t("模型配置文件不是有效的 JSON"));
  }
  const models = typeof parsed === "object" && parsed !== null && "models" in parsed
    ? (parsed as { models: unknown }).models
    : null;
  if (!Array.isArray(models)) throw new Error(t("模型配置文件中没有 models 列表"));
  if (!models.length) throw new Error(t("模型配置文件中没有模型"));
  if (!models.every((model) => typeof model === "object" && model !== null)) {
    throw new Error(t("模型配置文件格式不正确"));
  }
  return models as ModelInput[];
}

function errorText(cause: unknown) {
  return cause instanceof Error ? cause.message : String(cause);
}
