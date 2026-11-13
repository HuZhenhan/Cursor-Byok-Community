import type { Model, ModelInput, ModelType } from "../../shared/api";
import { modelPresets, presetEndpoint, trimTrailingSlash, type ModelPresetEntry } from "../../shared/utils/modelPresets";
import type { CursorModelDraft } from "./CursorModelEditor";

/** 「获取模型」结果的归属标识:接入参数变化后旧列表不再适用。 */
export function discoveryKey(model: ModelInput): string {
  return `${model.type}|${model.base_url.trim()}|${model.api_key.trim()}`;
}

const ACRONYMS: Record<string, string> = { ai: "AI", glm: "GLM", gpt: "GPT", llm: "LLM", oss: "OSS" };
const VERSION_TOKEN = /^\d+(?:\.\d+)*$/;

/** 由模型标识生成显示名:`claude-opus-5-5` → `Claude Opus 5.5`。 */
export function displayNameFromModelId(modelId: string): string {
  const tokens = modelId.trim().split(/[-_/\s]+/).filter(Boolean);
  if (!tokens.length) return "";
  let versionStart = tokens.length;
  while (versionStart > 0 && VERSION_TOKEN.test(tokens[versionStart - 1])) versionStart -= 1;
  const words = tokens.slice(0, versionStart).map((token) => ACRONYMS[token.toLowerCase()] ?? token.charAt(0).toUpperCase() + token.slice(1));
  const version = tokens.slice(versionStart).join(".");
  return [...words, version].filter(Boolean).join(" ");
}

/** 构造与具体模型标识无关的公共字段,单条保存与批量添加共用。 */
export function draftSharedInput(draft: CursorModelDraft): ModelInput {
  const model = {
    ...draft.model,
    base_url: draft.model.base_url.trim(),
    api_key: draft.model.api_key.trim(),
    openai_extra_params: parseObject(draft.openAIExtraParamsText, t("OpenAI 额外参数")),
    custom_headers: parseHeaders(draft.customHeadersText),
    anthropic_extra_params: parseObject(draft.anthropicExtraParamsText, t("Anthropic 额外参数")),
  };
  if (!model.base_url || !model.api_key) throw new Error(t("服务器地址或完整请求 URL、API Key 不能为空"));
  return model;
}

/** 校验并整理单条模型草稿;备注为选填,留空按空字符串保存。 */
export function draftInput(draft: CursorModelDraft): ModelInput {
  const model = {
    ...draftSharedInput(draft),
    display_name: draft.model.display_name.trim(),
    tooltip_data: draft.model.tooltip_data.trim(),
    model_id: draft.model.model_id.trim(),
  };
  if (!model.display_name || !model.model_id) throw new Error(t("模型名称和显示名称不能为空"));
  for (const [label, value] of [[t("上下文窗口 Token"), model.context_window_tokens], [t("最大输出 Token"), model.type === "openai" ? model.max_completion_tokens : model.anthropic_max_tokens], [t("思考预算 Token"), model.thinking_budget_tokens]] as const) {
    if (value !== null && (!Number.isSafeInteger(value) || value <= 0)) throw new Error(t("{label} 必须是大于 0 的整数", { label }));
  }
  return model;
}

/** 批量添加时按服务商端点匹配预设,沿用预设的显示名与 Token 上限。 */
export function presetModelEntry(type: ModelType, baseUrl: string, modelId: string): ModelPresetEntry | null {
  const base = trimTrailingSlash(baseUrl.trim());
  const preset = modelPresets.find((candidate) => trimTrailingSlash(presetEndpoint(candidate, type).baseUrl) === base);
  return preset?.models.find((entry) => entry.model_id === modelId) ?? null;
}

export function parseHeaders(text: string): Record<string, string> {
  const parsed = parseObject(text, t("自定义 Headers"));
  if (Object.values(parsed).some((value) => typeof value !== "string")) throw new Error(t("自定义 Headers 的值必须都是字符串"));
  return parsed as Record<string, string>;
}

export function parseObject(text: string, label: string): Record<string, unknown> {
  let parsed: unknown;
  try { parsed = JSON.parse(text || "{}"); } catch { throw new Error(t("{label} 必须是有效 JSON", { label })); }
  if (!parsed || Array.isArray(parsed) || typeof parsed !== "object") throw new Error(t("{label} 必须是 JSON 对象", { label }));
  return parsed as Record<string, unknown>;
}

/** 从模型配置中去除服务端生成的字段,得到可回写的输入。 */
export function modelInput(model: Model): ModelInput {
  const { model_hash: _hash, created_at_ms: _created, updated_at_ms: _updated, ...input } = model;
  return input;
}
