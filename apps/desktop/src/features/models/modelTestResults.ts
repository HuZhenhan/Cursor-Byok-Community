import type { CursorModelTestState } from "./CursorModelTestResult";

const STORAGE_KEY = "cursor-byok.model-test-results";

/** 读取上次保存的测速结果,刷新或重启后继续显示在模型条目上。 */
export function loadModelTestResults(): Map<string, CursorModelTestState> {
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "{}");
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) return new Map();
    return new Map(
      Object.entries(parsed as Record<string, unknown>)
        .filter((entry): entry is [string, CursorModelTestState] => isTestState(entry[1])),
    );
  } catch {
    return new Map();
  }
}

/** 覆盖单个模型的测速结果,重新测试时新结果替换旧结果。 */
export function saveModelTestResult(modelHash: string, state: CursorModelTestState) {
  const results = loadModelTestResults();
  results.set(modelHash, state);
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(Object.fromEntries(results)));
  } catch {
    // 存储不可用时结果仍保留在当前会话的内存状态中。
  }
}

function isTestState(value: unknown): value is CursorModelTestState {
  if (!value || typeof value !== "object") return false;
  const status = (value as { status?: unknown }).status;
  return status === "success" || status === "error" || status === "cancelled";
}
