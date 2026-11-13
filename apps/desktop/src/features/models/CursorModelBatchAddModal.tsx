import { useState } from "react";
import { api, type Model, type ModelInput } from "../../shared/api";
import { Button } from "../../shared/ui/Button";
import { Modal } from "../../shared/ui/Modal";
import { useMessage } from "../../shared/ui/message";
import { CursorModelBatchAdd } from "./CursorModelBatchAdd";
import { CursorModelConnectionFields } from "./CursorModelConnectionFields";
import { emptyCursorModelDraft, type CursorModelDraft } from "./CursorModelEditor";
import { discoveryKey, draftSharedInput, parseHeaders } from "./modelDraft";
import styles from "./CursorSettings.module.scss";

/** 常驻的批量添加入口:先读取模型列表,再逐项勾选添加。 */
export function CursorModelBatchAddModal({ models, busy, onClose, onAdd }: {
  models: Model[];
  busy: boolean;
  onClose: () => void;
  onAdd: (base: ModelInput, modelIds: string[]) => void;
}) {
  const [draft, setDraft] = useState<CursorModelDraft>(() => emptyCursorModelDraft());
  const message = useMessage();
  const [discovered, setDiscovered] = useState<{ key: string; models: string[] } | null>(null);
  const [discovering, setDiscovering] = useState(false);
  const canDiscover = Boolean(draft.model.base_url.trim() && draft.model.api_key.trim());
  const discoveredModels = discovered?.key === discoveryKey(draft.model) ? discovered.models : [];
  const existingModelIds = models
    .filter((model) => model.type === draft.model.type && model.base_url === draft.model.base_url.trim())
    .map((model) => model.model_id);

  const discover = async () => {
    setDiscovering(true);
    try {
      const result = await api.discoverModels({
        type: draft.model.type,
        base_url: draft.model.base_url.trim(),
        api_key: draft.model.api_key.trim(),
        custom_headers_enabled: draft.model.custom_headers_enabled,
        custom_headers: parseHeaders(draft.customHeadersText),
      });
      setDiscovered({ key: discoveryKey(draft.model), models: [...new Set(result.models)] });
    } catch (cause) {
      message(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setDiscovering(false);
    }
  };
  const add = (modelIds: string[]) => {
    try {
      onAdd(draftSharedInput(draft), modelIds);
    } catch (cause) {
      message(cause instanceof Error ? cause.message : String(cause));
    }
  };

  return <Modal open title={t("批量添加模型")} busy={busy || discovering} onClose={onClose} closeLabel={t("关闭")}>
    <div className={styles.editor}>
      <p className={styles.batchHint}>{t("填写接入参数并获取模型列表，然后勾选需要添加的模型。")}</p>
      <div className={styles.grid}>
        <CursorModelConnectionFields draft={draft} onChange={setDraft} />
      </div>
      <div className={styles.batchDiscover}>
        <Button className={styles.discoverButton} disabled={discovering || !canDiscover} onClick={() => void discover()}>{discovering ? t("获取中…") : t("获取模型")}</Button>
        {discoveredModels.length === 0 && <span className={styles.batchHint}>{t("服务器地址与 API Key 填写完整后才能读取模型列表。")}</span>}
      </div>
      {discoveredModels.length > 0 && <CursorModelBatchAdd models={discoveredModels} existingModelIds={existingModelIds} busy={busy} onAdd={add} />}
    </div>
  </Modal>;
}
