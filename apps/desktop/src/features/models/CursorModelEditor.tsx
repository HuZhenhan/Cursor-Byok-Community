import { useRef } from "react";
import type { ModelInput } from "../../shared/api";
import { defaultCustomHeadersText } from "../../shared/utils/modelDefaults";
import { modelPresets, presetEndpoint, trimTrailingSlash, type ModelPreset } from "../../shared/utils/modelPresets";
import { Button } from "../../shared/ui/Button";
import { FormField, TextInput } from "../../shared/ui/FormControls";
import { JsonEditor } from "../../shared/ui/JsonEditor";
import { Combobox, Select, type ComboboxHandle } from "../../shared/ui/Select";
import { Switch } from "../../shared/ui/Switch";
import { CursorContextPresets } from "./CursorContextPresets";
import { CursorModelConnectionFields } from "./CursorModelConnectionFields";
import { CursorPresetChips } from "./CursorPresetChips";
import { displayNameFromModelId } from "./modelDraft";
import styles from "./CursorSettings.module.scss";

export type CursorModelDraft = {
  providerId: string;
  model: ModelInput;
  openAIExtraParamsText: string;
  customHeadersText: string;
  anthropicExtraParamsText: string;
};

export const emptyCursorModelDraft = (): CursorModelDraft => ({
  providerId: "builtin/openai",
  model: {
    sort_order: 0,
    display_name: "",
    group_name: null,
    type: "openai",
    base_url: "",
    use_full_url: false,
    api_key: "",
    tooltip_data: "",
    model_id: "",
    reasoning_effort: null,
    openai_endpoint: "/v1/responses",
    openai_extra_params_enabled: false,
    openai_extra_params: {},
    custom_headers_enabled: false,
    custom_headers: {},
    anthropic_extra_params_enabled: false,
    anthropic_extra_params: {},
    context_window_tokens: null,
    max_completion_tokens: null,
    anthropic_max_tokens: null,
    anthropic_thinking_effort: "xhigh",
    thinking_budget_tokens: null,
  },
  openAIExtraParamsText: "{}",
  customHeadersText: defaultCustomHeadersText,
  anthropicExtraParamsText: "{}",
});

export function CursorModelEditor({ draft, modelOptions, discovering, onChange, onDiscover }: {
  draft: CursorModelDraft;
  modelOptions: string[];
  discovering: boolean;
  onChange: (draft: CursorModelDraft) => void;
  onDiscover: () => Promise<boolean>;
}) {
  const modelCombobox = useRef<ComboboxHandle>(null);
  const setModel = (patch: Partial<ModelInput>) => onChange({ ...draft, model: { ...draft.model, ...patch } });
  const numberValue = (value: string) => value === "" ? null : Math.trunc(Number(value));
  const canDiscover = Boolean(draft.model.base_url.trim() && draft.model.api_key.trim());
  // 选中预设后，把该服务商已知的模型 id 并入下拉，方便直接选（仍可用「获取模型」发现）
  const presetModelOptions = modelPresets
    .filter((preset) => trimTrailingSlash(presetEndpoint(preset, draft.model.type).baseUrl) === trimTrailingSlash(draft.model.base_url.trim()))
    .flatMap((preset) => preset.models.map((item) => item.model_id));
  const combinedOptions = [...new Set([...modelOptions, ...presetModelOptions])];
  const discoverModels = async () => {
    if (await onDiscover()) modelCombobox.current?.openAll();
  };
  const applyPreset = (preset: ModelPreset) => {
    const endpoint = presetEndpoint(preset, draft.model.type);
    const first = preset.models[0];
    // 切到别家服务商时清空 API Key（不同家的 Key 不能串用）；同一家内切换协议则保留
    const currentBase = trimTrailingSlash(draft.model.base_url.trim());
    const sameProvider = [preset.endpoints.anthropic, preset.endpoints.openai]
      .some((candidate) => trimTrailingSlash(candidate.baseUrl) === currentBase);
    onChange({
      ...draft,
      model: {
        ...draft.model,
        base_url: endpoint.baseUrl,
        use_full_url: endpoint.useFullUrl,
        openai_endpoint: draft.model.type === "openai" ? endpoint.openaiEndpoint : draft.model.openai_endpoint,
        custom_headers_enabled: endpoint.customHeaders !== null,
        custom_headers: endpoint.customHeaders ? { ...endpoint.customHeaders } : {},
        api_key: sameProvider ? draft.model.api_key : "",
        model_id: first?.model_id ?? draft.model.model_id,
        display_name: first?.display_name ?? draft.model.display_name,
        tooltip_data: !draft.model.tooltip_data.trim() || draft.model.tooltip_data === t("备注") ? preset.name : draft.model.tooltip_data,
        context_window_tokens: first?.context_window_tokens ?? draft.model.context_window_tokens,
        ...(draft.model.type === "openai"
          ? { max_completion_tokens: first?.max_output_tokens ?? draft.model.max_completion_tokens }
          : { anthropic_max_tokens: first?.max_output_tokens ?? draft.model.anthropic_max_tokens }),
      },
      customHeadersText: endpoint.customHeaders ? JSON.stringify(endpoint.customHeaders, null, 2) : draft.customHeadersText,
    });
  };
  return <div className={styles.editor}>
    <CursorPresetChips type={draft.model.type} baseUrl={draft.model.base_url} onPick={applyPreset} />
    <div className={styles.grid}>
      <CursorModelConnectionFields draft={draft} onChange={onChange} />

      <FormField label={t("模型名称")} hint={t("可以直接输入模型标识，也可以读取接口返回的模型列表。")}><Combobox ref={modelCombobox} value={draft.model.model_id} options={combinedOptions} placeholder="gpt-5" append={<Button className={styles.discoverButton} disabled={discovering || !canDiscover} onClick={() => void discoverModels()}>{discovering ? t("获取中…") : t("获取模型")}</Button>} onChange={(model_id) => setModel({ model_id, display_name: autoDisplayName(draft, model_id) })} /></FormField>
      <FormField label={t("显示名称")} hint={t("默认按模型标识自动生成，可手动修改。")}> <TextInput placeholder={t("例如：主力模型")} value={draft.model.display_name} onChange={(event) => setModel({ display_name: event.target.value })} /></FormField>
      <FormField className={styles.fullWidth} label={t("备注")} hint={t("选填，显示在 Cursor 模型说明中。")}> <TextInput placeholder={t("请输入模型备注")} value={draft.model.tooltip_data} onChange={(event) => setModel({ tooltip_data: event.target.value })} /></FormField>

      <div className={styles.contextField}>
        <FormField label={t("上下文窗口 Token")} hint={t("留空时使用默认值；可选择常用档位，也可以填写自定义值。")}>
          <TextInput type="number" min={1} step={1} placeholder={t("留空使用默认值")} value={draft.model.context_window_tokens ?? ""} onChange={(event) => setModel({ context_window_tokens: numberValue(event.target.value) })} />
        </FormField>
        <CursorContextPresets value={draft.model.context_window_tokens} onChange={(context_window_tokens) => setModel({ context_window_tokens })} />
      </div>
      {draft.model.type === "openai" ? <>
        <FormField label={t("最大输出 Token")} hint={t("留空时使用默认值。")}> <TextInput type="number" min={1} step={1} placeholder={t("留空使用默认值")} value={draft.model.max_completion_tokens ?? ""} onChange={(event) => setModel({ max_completion_tokens: numberValue(event.target.value) })} /></FormField>
        <FormField label={t("推理强度")}> <Select ariaLabel={t("推理强度")} value={draft.model.reasoning_effort ?? ""} options={effortOptions(true)} onChange={(value) => setModel({ reasoning_effort: value || null })} /></FormField>
      </> : <>
        <FormField label={t("最大输出 Token")} hint={t("留空时使用默认值。")}> <TextInput type="number" min={1} step={1} placeholder={t("留空使用默认值")} value={draft.model.anthropic_max_tokens ?? ""} onChange={(event) => setModel({ anthropic_max_tokens: numberValue(event.target.value) })} /></FormField>
        <FormField label={t("思考强度")}> <Select ariaLabel={t("思考强度")} value={draft.model.anthropic_thinking_effort ?? "xhigh"} options={effortOptions(false)} onChange={(anthropic_thinking_effort) => setModel({ anthropic_thinking_effort })} /></FormField>
        <FormField label={t("思考预算 Token")} hint={t("留空时使用 adaptive thinking。")}> <TextInput type="number" min={1} step={1} placeholder={t("留空使用 adaptive thinking")} value={draft.model.thinking_budget_tokens ?? ""} onChange={(event) => setModel({ thinking_budget_tokens: numberValue(event.target.value) })} /></FormField>
      </>}

      <ToggleJsonField
        label={t("自定义 Headers")}
        enabled={draft.model.custom_headers_enabled}
        text={draft.customHeadersText}
        onEnabledChange={(custom_headers_enabled) => setModel({ custom_headers_enabled })}
        onTextChange={(customHeadersText) => onChange({ ...draft, customHeadersText })}
      />
      {draft.model.type === "openai" ? <ToggleJsonField
        label={t("OpenAI 额外参数")}
        enabled={draft.model.openai_extra_params_enabled}
        text={draft.openAIExtraParamsText}
        onEnabledChange={(openai_extra_params_enabled) => setModel({ openai_extra_params_enabled })}
        onTextChange={(openAIExtraParamsText) => onChange({ ...draft, openAIExtraParamsText })}
      /> : <ToggleJsonField
        label={t("Anthropic 额外参数")}
        enabled={draft.model.anthropic_extra_params_enabled}
        text={draft.anthropicExtraParamsText}
        onEnabledChange={(anthropic_extra_params_enabled) => setModel({ anthropic_extra_params_enabled })}
        onTextChange={(anthropicExtraParamsText) => onChange({ ...draft, anthropicExtraParamsText })}
      />}
    </div>
  </div>;
}

function ToggleJsonField({ label, enabled, text, onEnabledChange, onTextChange }: {
  label: string;
  enabled: boolean;
  text: string;
  onEnabledChange: (enabled: boolean) => void;
  onTextChange: (text: string) => void;
}) {
  return <div className={`${styles.fullWidth} ${styles.jsonOption}`}>
    <label><span>{label}</span><Switch label={label} checked={enabled} onChange={onEnabledChange} /></label>
    {enabled && <JsonEditor ariaLabel={label} value={text} onChange={onTextChange} />}
  </div>;
}

/** 显示名为空或仍是上一个模型标识自动生成的值时，跟随模型标识自动更新。 */
function autoDisplayName(draft: CursorModelDraft, modelId: string): string {
  const current = draft.model.display_name.trim();
  if (!current || current === displayNameFromModelId(draft.model.model_id)) return displayNameFromModelId(modelId);
  return draft.model.display_name;
}

function effortOptions(optional: boolean) {
  return [
    ...(optional ? [{ value: "", label: t("不设置") }] : []),
    { value: "low", label: "Low" },
    { value: "medium", label: "Medium" },
    { value: "high", label: "High" },
    { value: "xhigh", label: "Extra High" },
    { value: "max", label: "Max" },
  ];
}
