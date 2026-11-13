import type { ModelInput, ModelType } from "../../shared/api";
import { modelPresets, presetEndpoint, trimTrailingSlash } from "../../shared/utils/modelPresets";
import { Checkbox } from "../../shared/ui/Checkbox";
import { FormField, SecretTextInput, TextInput } from "../../shared/ui/FormControls";
import { Select } from "../../shared/ui/Select";
import { claudeIcon, openAiIcon } from "../../shared/ui/icons";
import type { CursorModelDraft } from "./CursorModelEditor";
import styles from "./CursorSettings.module.scss";

/** 模型接入参数（类型 / 协议 / 地址 / 密钥），单条编辑与批量添加共用。 */
export function CursorModelConnectionFields({ draft, onChange }: {
  draft: CursorModelDraft;
  onChange: (draft: CursorModelDraft) => void;
}) {
  const setModel = (patch: Partial<ModelInput>) => onChange({ ...draft, model: { ...draft.model, ...patch } });
  const setType = (type: ModelType) => {
    // 切换协议类型时，若当前地址命中某预设的另一协议端点，自动换到该预设对应协议的端点，
    // 避免出现「类型是 Anthropic、URL 却是 OpenAI chat/completions」的错配
    const other: ModelType = type === "anthropic" ? "openai" : "anthropic";
    const preset = modelPresets.find((candidate) => trimTrailingSlash(presetEndpoint(candidate, other).baseUrl) === trimTrailingSlash(draft.model.base_url.trim()));
    const endpoint = preset ? presetEndpoint(preset, type) : null;
    onChange({
      ...draft,
      providerId: `builtin/${type}`,
      model: {
        ...draft.model,
        type,
        ...(endpoint ? {
          base_url: endpoint.baseUrl,
          use_full_url: endpoint.useFullUrl,
          custom_headers_enabled: endpoint.customHeaders !== null,
          custom_headers: endpoint.customHeaders ? { ...endpoint.customHeaders } : {},
        } : {}),
        openai_endpoint: type === "openai" ? (endpoint?.openaiEndpoint || draft.model.openai_endpoint || "/v1/responses") : "",
        anthropic_thinking_effort: type === "anthropic" ? draft.model.anthropic_thinking_effort || "xhigh" : null,
      },
      customHeadersText: endpoint?.customHeaders ? JSON.stringify(endpoint.customHeaders, null, 2) : draft.customHeadersText,
    });
  };
  const requestUrlPlaceholder = draft.model.use_full_url
    ? draft.model.type === "anthropic"
      ? "https://api.anthropic.com/v1/messages"
      : draft.model.openai_endpoint === "/v1/chat/completions"
        ? "https://api.openai.com/v1/chat/completions"
        : "https://api.openai.com/v1/responses"
    : draft.model.type === "anthropic"
      ? "https://api.anthropic.com"
      : "https://api.openai.com";
  const providerOptions = [
    { value: "builtin/openai", label: "OpenAI", icon: openAiIcon },
    { value: "builtin/anthropic", label: "Anthropic", icon: claudeIcon },
  ];
  const setProvider = (providerId: string) => {
    if (providerId === "builtin/openai") setType("openai");
    if (providerId === "builtin/anthropic") setType("anthropic");
  };

  return <>
    <FormField label={t("模型类型")}><Select ariaLabel={t("模型类型")} value={draft.providerId} options={providerOptions} onChange={setProvider} /></FormField>
    {draft.model.type === "openai" && <FormField label={t("请求协议")} hint={t("只决定请求与响应的格式，不会改变请求地址。")}> <Select ariaLabel={t("请求协议")} value={draft.model.openai_endpoint} options={[
      { value: "/v1/responses", label: "Responses API" },
      { value: "/v1/chat/completions", label: "Chat Completions API" },
    ]} onChange={(openai_endpoint) => setModel({ openai_endpoint })} /></FormField>}
    <div className={styles.urlField}>
      <FormField label={draft.model.use_full_url ? t("完整请求 URL") : t("服务器地址")} hint={draft.model.use_full_url ? t("系统会原样使用此地址，不追加或修改请求路径。") : t("系统会根据请求协议自动追加标准端点路径。")}> <TextInput placeholder={requestUrlPlaceholder} value={draft.model.base_url} onChange={(event) => setModel({ base_url: event.target.value })} /></FormField>
      <Checkbox checked={draft.model.use_full_url} label={t("使用完整请求地址")} onChange={(use_full_url) => setModel({ use_full_url })} />
    </div>
    <FormField label="API Key" hint={t("访问模型服务所需的密钥。")}> <SecretTextInput placeholder="sk-xxxxxx" autoComplete="off" value={draft.model.api_key} onChange={(event) => setModel({ api_key: event.target.value })} /></FormField>
  </>;
}
