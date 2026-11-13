import { useMemo } from "react";
import cursorIconUrl from "../../shared/assets/icons/cursor.svg";
import { pluginText, type TabMode, type TabSettings } from "../../shared/api";
import { useI18n } from "../../i18n/store";
import { useAppStore } from "../../shared/store/appStore";
import { Button } from "../../shared/ui/Button";
import { TextInput } from "../../shared/ui/FormControls";
import { Icon } from "../../shared/ui/Icon";
import { ModelSelect, type ModelSelectOption } from "../../shared/ui/ModelSelect";
import { claudeIcon, flatColorOrganizationIcon, openAiIcon } from "../../shared/ui/icons";
import { Select } from "../../shared/ui/Select";
import { TitledCard } from "../../shared/ui/TitledCard";
import { modelProviderName } from "../../shared/utils/modelProvider";
import styles from "./TabSettingsCard.module.scss";

export function TabSettingsCard({
  settings,
  draft,
  editing,
  saving,
  onDraftChange,
  onEdit,
  onCancel,
  onSave,
}: {
  settings: TabSettings | null;
  draft: TabSettings;
  editing: boolean;
  saving: boolean;
  onDraftChange: (settings: TabSettings) => void;
  onEdit: () => void;
  onCancel: () => void;
  onSave: () => void;
}) {
  const { models, plugins } = useAppStore();
  const { locale } = useI18n();
  const modeLabel = (mode: TabMode) => {
    if (mode === "local") return t("自定义模型");
    if (mode === "direct") return t("官方直连");
    return t("自定义服务");
  };
  const modelOptions = useMemo(() => {
    const options: ModelSelectOption[] = [];
    for (const model of models) {
      options.push({
        value: model.model_hash,
        label: model.display_name && model.display_name !== model.model_id
          ? `${model.display_name}（${model.model_id}）`
          : model.display_name || model.model_id,
        group: modelProviderName(model),
        icon: model.type === "anthropic" ? claudeIcon : openAiIcon,
      });
    }
    for (const plugin of plugins) {
      for (const provider of plugin.providers) {
        if (!provider.configured) continue;
        const group = pluginText(provider.displayName, locale) || plugin.name;
        for (const model of provider.models.filter((model) => model.enabled)) {
          options.push({
            value: model.id,
            label: model.displayName,
            group,
            iconSrc: model.icon || undefined,
            icon: model.icon ? undefined : flatColorOrganizationIcon,
          });
        }
      }
    }
    if (draft.model_id && !options.some((option) => option.value === draft.model_id)) {
      options.push({ value: draft.model_id, label: draft.model_id, group: "Cursor" });
    }
    return options;
  }, [draft.model_id, locale, models, plugins]);
  const selectedModelLabel = modelOptions.find((option) => option.value === settings?.model_id)?.label
    ?? settings?.model_id
    ?? t("未选择");
  const action = editing ? (
    <div className={styles.actionGroup}>
      <Button size="small" disabled={saving} onClick={onCancel}>{t("取消")}</Button>
      <Button variant="primary" size="small" disabled={saving} onClick={onSave}>{saving ? t("保存中…") : t("保存")}</Button>
    </div>
  ) : (
    <button type="button" className={styles.headerAction} disabled={!settings} onClick={onEdit}>{t("编辑")}</button>
  );

  return <TitledCard
    title={<div className={styles.title}><Icon src={cursorIconUrl} size="1.1em" /><span>{t("TAB 设置")}</span></div>}
    action={action}
  >
    <div className={styles.content}>
      {editing ? <>
        <div className={styles.row}>
          <div className={styles.description}>
            <strong>{t("TAB 选择")}</strong>
            <small>{t("控制 Cursor TAB 相关接口的连接方式。")}</small>
          </div>
          <div className={styles.control}><Select
            value={draft.mode}
            ariaLabel={t("TAB 选择")}
            options={[
              { value: "local", label: t("自定义模型") },
              { value: "direct", label: t("官方直连") },
              { value: "custom", label: t("自定义服务") },
            ]}
            onChange={(mode) => onDraftChange({ ...draft, mode: mode as TabMode })}
          /></div>
        </div>
        {draft.mode === "local" && <div className={styles.row}>
          <div className={styles.description}>
            <strong>{t("TAB 模型")}</strong>
            <small>{t("使用所选模型生成补全，建议选择响应较快的模型。")}</small>
          </div>
          <div className={styles.control}><ModelSelect
            mode="single"
            value={draft.model_id}
            options={modelOptions}
            label={t("TAB 模型")}
            onChange={(modelId) => onDraftChange({ ...draft, model_id: modelId })}
          /></div>
        </div>}
        {draft.mode === "custom" && <div className={styles.row}>
          <div className={styles.description}>
            <strong>{t("TAB 服务地址")}</strong>
            <small>{t("原接口路径会追加到此服务地址。")}</small>
          </div>
          <div className={styles.control}><TextInput
            value={draft.address}
            placeholder="https://tab.example.com"
            aria-label={t("TAB 服务地址")}
            onChange={(event) => onDraftChange({ ...draft, address: event.target.value })}
            onKeyDown={(event) => { if (event.key === "Enter") onSave(); }}
          /></div>
        </div>}
      </> : <>
        <div className={styles.row}>
          <strong>{t("TAB 选择")}</strong>
          <span className={styles.value}>{settings ? modeLabel(settings.mode) : t("加载中…")}</span>
        </div>
        {settings?.mode === "local" && <div className={styles.row}>
          <strong>{t("TAB 模型")}</strong>
          <span className={styles.value}>{selectedModelLabel}</span>
        </div>}
        {settings?.mode === "custom" && <div className={styles.row}>
          <strong>{t("TAB 服务地址")}</strong>
          <span className={styles.value}>{settings.address}</span>
        </div>}
      </>}
    </div>
  </TitledCard>;
}
