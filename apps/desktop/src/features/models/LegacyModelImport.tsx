import { useState, type ReactNode } from "react";
import { api, type LegacyModelImportPreview } from "../../shared/api";
import { appStore } from "../../shared/store/appStore";
import { useMessage } from "../../shared/ui/message";
import { ModelImportDialog } from "./ModelImportDialog";

type LegacyModelImportControl = {
  busy: boolean;
  previewing: boolean;
  open: () => void;
};

export function LegacyModelImport({ children }: { children: (control: LegacyModelImportControl) => ReactNode }) {
  const message = useMessage();
  const [preview, setPreview] = useState<LegacyModelImportPreview | null>(null);
  const [previewing, setPreviewing] = useState(false);
  const [importing, setImporting] = useState(false);

  const open = async () => {
    try {
      setPreviewing(true);
      setPreview(await api.previewV0049Models());
    } catch (cause) {
      message(errorText(cause));
    } finally {
      setPreviewing(false);
    }
  };

  const confirm = async () => {
    try {
      setImporting(true);
      const result = await appStore.importV0049Models();
      if (!result) {
        const error = appStore.getSnapshot().error;
        if (error) message(error);
        return;
      }
      setPreview(null);
      message(result.imported > 0
        ? t("导入完成：新增 {imported} 个模型，跳过 {skipped} 个已存在模型", { imported: result.imported, skipped: result.skipped })
        : t("配置中的 {count} 个模型均已存在，无需重复导入", { count: result.skipped }));
    } catch (cause) {
      message(errorText(cause));
    } finally {
      setImporting(false);
    }
  };

  return <>
    {children({ busy: previewing || importing, previewing, open: () => void open() })}
    <ModelImportDialog
      open={preview !== null}
      title={t("确认导入旧版模型配置")}
      source={preview?.source ?? ""}
      items={(preview?.models ?? []).map((model) => ({
        key: model.model_hash,
        display_name: model.display_name,
        model_id: model.model_id,
        type: model.type,
        existing: model.existing,
      }))}
      total={preview?.total ?? 0}
      newModels={preview?.new_models ?? 0}
      existingModels={preview?.existing_models ?? 0}
      hint={t("重复导入相同配置不会创建重复模型；已经存在的模型会自动跳过。")}
      busy={importing}
      onCancel={() => setPreview(null)}
      onConfirm={() => void confirm()}
    />
  </>;
}

function errorText(cause: unknown) {
  return cause instanceof Error ? cause.message : String(cause);
}
