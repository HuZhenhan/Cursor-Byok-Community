import { useCallback, useEffect, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { api, configuredPluginModels, type Model, type ModelInput } from "../../shared/api";
import { CursorCaGate, CursorCaProvider, CursorModelGate, CursorModelProvider } from "./CursorGates";
import { CursorModelCards, cursorModelGroups, type CursorModelGroup, type CursorModelGrouping, type CursorModelView } from "./CursorModelCards";
import { CursorModelBatchAddModal } from "./CursorModelBatchAddModal";
import { CursorModelEditor, emptyCursorModelDraft, type CursorModelDraft } from "./CursorModelEditor";
import { CursorModelTestResult, type CursorModelTestState } from "./CursorModelTestResult";
import { displayNameFromModelId, draftInput, modelInput, parseHeaders, presetModelEntry } from "./modelDraft";
import { loadModelTestResults, saveModelTestResult } from "./modelTestResults";
import styles from "./CursorSettings.module.scss";
import { PageContent } from "../../shell/layout/PageContent";
import { LegacyModelImport } from "./LegacyModelImport";
import { ConfirmDialog } from "../../shared/ui/ConfirmDialog";
import { FormField, SecretTextInput, TextInput } from "../../shared/ui/FormControls";
import controls from "../../shared/ui/Controls.module.scss";
import { Icon } from "../../shared/ui/Icon";
import { Modal } from "../../shared/ui/Modal";
import { Switch } from "../../shared/ui/Switch";
import { TooltipTrigger } from "../../shared/ui/TooltipTrigger";
import { addIcon } from "../../shared/ui/icons";
import { useMessage } from "../../shared/ui/message";
import { PageActions } from "../../shell/PageActions";
import { appStore, useAppStore } from "../../shared/store/appStore";

export function CursorSettingsPage() {
  const { models, cursorHarness, cursorBusy, plugins } = useAppStore();
  const navigate = useNavigate();
  const message = useMessage();
  const [draft, setDraft] = useState<CursorModelDraft | null>(null);
  const [editing, setEditing] = useState<Model | null>(null);
  const [modelOptions, setModelOptions] = useState<string[]>([]);
  const [discovering, setDiscovering] = useState(false);
  const [caCommand, setCaCommand] = useState<string | null>(null);
  const [waitingForCaRefresh, setWaitingForCaRefresh] = useState(false);
  const [deleting, setDeleting] = useState<Model | null>(null);
  const [confirmDisableTakeover, setConfirmDisableTakeover] = useState(false);
  const [testingModelHashes, setTestingModelHashes] = useState<Set<string>>(() => new Set());
  const [selectedModelHashes, setSelectedModelHashes] = useState<Set<string>>(() => new Set());
  const [modelTestResults, setModelTestResults] = useState<Map<string, CursorModelTestState>>(() => loadModelTestResults());
  const [savingAndTesting, setSavingAndTesting] = useState(false);
  const [batchTesting, setBatchTesting] = useState(false);
  const [batchAdding, setBatchAdding] = useState(false);
  const [batchDeleting, setBatchDeleting] = useState(false);
  const [confirmBatchDelete, setConfirmBatchDelete] = useState(false);
  const [batchAddOpen, setBatchAddOpen] = useState(false);
  const [view, setView] = useState<CursorModelView>("table");
  const [grouping, setGrouping] = useState<CursorModelGrouping>("none");
  const [settingsGroup, setSettingsGroup] = useState<CursorModelGroup | null>(null);
  const [groupNameDraft, setGroupNameDraft] = useState("");
  const [groupBaseUrlDraft, setGroupBaseUrlDraft] = useState("");
  const [groupApiKeyDraft, setGroupApiKeyDraft] = useState("");
  const [groupSettingsBusy, setGroupSettingsBusy] = useState(false);
  const activeModelTests = useRef(new Map<string, { testId: string; controller: AbortController; cancelling: boolean }>());
  const caReady = cursorHarness?.ca === "ready";
  const cursorTakenOver = cursorHarness?.settings_applied ?? false;
  const takeoverLabel = cursorTakenOver ? t("关闭接管Cursor") : t("开启接管Cursor");
  const pluginModels = configuredPluginModels(plugins);
  const testTargets = [
    ...models.map((model) => ({ model_hash: model.model_hash, display_name: model.display_name })),
    ...pluginModels.map((model) => ({ model_hash: model.id, display_name: model.displayName })),
  ];
  const providerGroups = cursorModelGroups(models, "provider");
  const typeGroups = cursorModelGroups(models, "type");
  const canGroupByProvider = providerGroups.length > 1;
  const canGroupByType = typeGroups.length > 1;
  const selectedTargets = testTargets.filter((target) => selectedModelHashes.has(target.model_hash));
  const selectedModels = models.filter((model) => selectedModelHashes.has(model.model_hash));

  useEffect(() => {
    if ((grouping === "provider" && !canGroupByProvider) || (grouping === "type" && !canGroupByType)) {
      setGrouping("none");
    }
  }, [canGroupByProvider, canGroupByType, grouping]);

  useEffect(() => {
    if (caCommand) void api.copyCursorText(caCommand);
  }, [caCommand]);

  const initializeCa = async () => {
    const status = await appStore.initializeCursorCa();
    if (status?.ca === "untrusted" && status.ca_install_command) setCaCommand(status.ca_install_command);
  };
  const openNew = () => {
    const next = emptyCursorModelDraft();
    next.model.sort_order = models.length + 1;
    setEditing(null);
    setModelOptions([]);
    setDraft(next);
  };
  const openEdit = (model: Model) => {
    setEditing(model);
    setModelOptions([model.model_id]);
    setDraft({
      providerId: `builtin/${model.type}`,
      model: modelInput(model),
      openAIExtraParamsText: JSON.stringify(model.openai_extra_params, null, 2),
      customHeadersText: JSON.stringify(model.custom_headers, null, 2),
      anthropicExtraParamsText: JSON.stringify(model.anthropic_extra_params, null, 2),
    });
  };
  const discover = async (): Promise<boolean> => {
    if (!draft) return false;
    setDiscovering(true);
    try {
      const custom_headers = parseHeaders(draft.customHeadersText);
      const result = await api.discoverModels({
        type: draft.model.type,
        base_url: draft.model.base_url.trim(),
        api_key: draft.model.api_key.trim(),
        custom_headers_enabled: draft.model.custom_headers_enabled,
        custom_headers,
      });
      setModelOptions([...new Set(result.models)]);
      return true;
    } catch (cause) {
      message(errorText(cause));
      return false;
    } finally {
      setDiscovering(false);
    }
  };
  const persist = async (): Promise<Model | null> => {
    if (!draft) return null;
    const input = draftInput(draft);
    if (editing) return appStore.updateCursorModel(editing.model_hash, input);
    return (await appStore.createModels([input]))?.[0] ?? null;
  };
  const save = async () => {
    try {
      if (await persist()) {
        setDraft(null);
        setEditing(null);
      }
    } catch (cause) {
      message(errorText(cause));
    }
  };
  const cancelModelTest = async (modelHash: string) => {
    const active = activeModelTests.current.get(modelHash);
    if (!active || active.cancelling) return;
    active.cancelling = true;
    active.controller.abort();
    try {
      await api.cancelModelTest(modelHash, active.testId);
    } catch (cause) {
      message(t("取消测试失败：{error}", { error: errorText(cause) }), { duration: 5000 });
    }
  };
  const cancelAllModelTests = async () => {
    await Promise.all([...activeModelTests.current.keys()].map((modelHash) => cancelModelTest(modelHash)));
  };
  const recordTestResult = (modelHash: string, state: CursorModelTestState) => {
    setModelTestResults((current) => new Map(current).set(modelHash, state));
    saveModelTestResult(modelHash, state);
  };
  const testModel = async (model: { model_hash: string; display_name: string }, notify = true): Promise<"success" | "failure" | "cancelled"> => {
    if (activeModelTests.current.has(model.model_hash)) {
      await cancelModelTest(model.model_hash);
      return "cancelled";
    }
    const active = { testId: crypto.randomUUID(), controller: new AbortController(), cancelling: false };
    activeModelTests.current.set(model.model_hash, active);
    setTestingModelHashes((current) => new Set(current).add(model.model_hash));
    try {
      const result = await api.testModel(model.model_hash, active.testId, active.controller.signal);
      recordTestResult(model.model_hash, { status: "success", result });
      if (notify) message(t("模型 {model} 连通性测试成功（{duration} ms）", { model: model.display_name, duration: result.duration_ms }));
      return "success";
    } catch (cause) {
      if (active.cancelling || active.controller.signal.aborted) {
        recordTestResult(model.model_hash, { status: "cancelled" });
        return "cancelled";
      }
      const error = errorText(cause);
      recordTestResult(model.model_hash, { status: "error", error });
      if (notify) message(t("连通性测试失败：{error}", { error }), { duration: 5000 });
      return "failure";
    } finally {
      if (activeModelTests.current.get(model.model_hash) === active) activeModelTests.current.delete(model.model_hash);
      setTestingModelHashes((current) => {
        const next = new Set(current);
        next.delete(model.model_hash);
        return next;
      });
    }
  };
  const saveAndTest = async () => {
    setSavingAndTesting(true);
    let saved: Model | null = null;
    try {
      saved = await persist();
    } catch (cause) {
      message(errorText(cause));
    } finally {
      setSavingAndTesting(false);
    }
    if (!saved) return;
    setEditing(saved);
    await testModel(saved);
    await appStore.refresh();
  };
  const runModelTests = async (targets: typeof testTargets) => {
    if (!targets.length || batchTesting) return;
    setBatchTesting(true);
    try {
      const results = await Promise.all(targets.map((model) => testModel(model, false)));
      const successful = results.filter((result) => result === "success").length;
      const failed = results.filter((result) => result === "failure").length;
      const cancelled = results.filter((result) => result === "cancelled").length;
      message(cancelled > 0
        ? t("连通性测试已取消：成功 {successful}，失败 {failed}", { successful, failed })
        : failed === 0
          ? t("全部 {count} 个模型连通性测试成功", { count: targets.length })
          : t("连通性测试完成：成功 {successful}，失败 {failed}", { successful, failed }),
      { duration: failed === 0 && cancelled === 0 ? 2400 : 5000 });
    } finally {
      setBatchTesting(false);
    }
  };
  const toggleModelSelection = (modelHash: string) => {
    setSelectedModelHashes((current) => {
      const next = new Set(current);
      if (next.has(modelHash)) next.delete(modelHash);
      else next.add(modelHash);
      return next;
    });
  };
  const setModelSelection = (modelHashes: string[], selected: boolean) => {
    setSelectedModelHashes((current) => {
      const next = new Set(current);
      for (const modelHash of modelHashes) {
        if (selected) next.add(modelHash);
        else next.delete(modelHash);
      }
      return next;
    });
  };
  const deleteSelectedModels = async () => {
    setConfirmBatchDelete(false);
    const targets = selectedModels.map((model) => model.model_hash);
    if (!targets.length) return;
    setBatchDeleting(true);
    try {
      for (const modelHash of targets) await api.deleteModel(modelHash);
      await appStore.refresh();
      setSelectedModelHashes((current) => {
        const next = new Set(current);
        for (const modelHash of targets) next.delete(modelHash);
        return next;
      });
      message(t("已删除 {count} 个模型", { count: targets.length }));
    } catch (cause) {
      message(errorText(cause));
    } finally {
      setBatchDeleting(false);
    }
  };
  const addBatchModels = async (base: ModelInput, modelIds: string[]) => {
    if (!modelIds.length) return;
    const existing = new Set(models
      .filter((model) => model.type === base.type && model.base_url === base.base_url)
      .map((model) => model.model_id));
    const pending = modelIds.filter((modelId) => !existing.has(modelId));
    if (!pending.length) {
      message(t("所选模型都已添加"));
      return;
    }
    setBatchAdding(true);
    try {
      const created = await appStore.createModels(pending.map((modelId, index) => {
        const preset = presetModelEntry(base.type, base.base_url, modelId);
        return {
          ...base,
          sort_order: models.length + index + 1,
          model_id: modelId,
          display_name: preset?.display_name ?? displayNameFromModelId(modelId),
          context_window_tokens: base.context_window_tokens ?? preset?.context_window_tokens ?? null,
          ...(base.type === "openai"
            ? { max_completion_tokens: base.max_completion_tokens ?? preset?.max_output_tokens ?? null }
            : { anthropic_max_tokens: base.anthropic_max_tokens ?? preset?.max_output_tokens ?? null }),
        };
      }));
      message(created
        ? t("已添加 {count} 个模型", { count: created.length })
        : appStore.getSnapshot().error || t("批量添加失败"));
    } finally {
      setBatchAdding(false);
    }
  };
  const duplicateModel = async (model: Model) => {
    const names = new Set(models.map((item) => item.display_name));
    const baseName = t("{name} 副本", { name: model.display_name });
    let displayName = baseName;
    let suffix = 2;
    while (names.has(displayName)) {
      displayName = `${baseName} ${suffix}`;
      suffix += 1;
    }
    const created = await appStore.createModels([{
      ...modelInput(model),
      sort_order: models.length + 1,
      display_name: displayName,
    }]);
    if (created) message(t("模型已复制"));
  };
  const openGroupSettings = (group: CursorModelGroup) => {
    setGroupNameDraft(group.models.find((model) => model.group_name?.trim())?.group_name?.trim() ?? "");
    setGroupBaseUrlDraft(sharedValue(group.models.map((model) => model.base_url)) ?? "");
    setGroupApiKeyDraft(sharedValue(group.models.map((model) => model.api_key)) ?? "");
    setSettingsGroup(group);
  };
  const saveGroupSettings = async () => {
    if (!settingsGroup) return;
    const group_name = groupNameDraft.trim() || null;
    const base_url = groupBaseUrlDraft.trim();
    const api_key = groupApiKeyDraft.trim();
    setGroupSettingsBusy(true);
    try {
      for (const model of settingsGroup.models) {
        const input: ModelInput = {
          ...modelInput(model),
          group_name,
          ...(base_url ? { base_url } : {}),
          ...(api_key ? { api_key } : {}),
        };
        if (input.group_name === (model.group_name ?? null)
          && input.base_url === model.base_url
          && input.api_key === model.api_key) continue;
        await api.updateModel(model.model_hash, input);
      }
      await appStore.refresh();
      setSettingsGroup(null);
    } catch (cause) {
      message(errorText(cause));
    } finally {
      setGroupSettingsBusy(false);
    }
  };
  const reorderModels = useCallback(async (modelHashes: string[]) => {
    if (!await appStore.reorderCursorModels(modelHashes)) {
      message(appStore.getSnapshot().error || t("排序失败"));
    }
  }, [message]);

  const listToolbarVisible = models.length > 0 || pluginModels.length > 0;
  const listToolbar = <div className={styles.listToolbar}>
    {testTargets.length > 0 && <div className={styles.groupActions} role="group" aria-label={t("展示方式")}>
      <button type="button" aria-pressed={view === "table"} onClick={() => setView("table")}>{t("表格")}</button>
      <button type="button" aria-pressed={view === "grid"} onClick={() => setView("grid")}>{t("平铺")}</button>
    </div>}
    {view === "table" && testTargets.length > 0 && (canGroupByProvider || canGroupByType) && <div className={styles.groupActions} role="group" aria-label={t("分组方式")}>
      <button type="button" aria-pressed={grouping === "none"} onClick={() => setGrouping("none")}>{t("不分组")}</button>
      {canGroupByProvider && <button type="button" aria-pressed={grouping === "provider"} onClick={() => setGrouping("provider")}>{t("按供应商")}</button>}
      {canGroupByType && <button type="button" aria-pressed={grouping === "type"} onClick={() => setGrouping("type")}>{t("按类型")}</button>}
    </div>}
    <div className={styles.groupActions} role="group" aria-label={t("批量操作")}>
      <button type="button" disabled={cursorBusy || testTargets.length === 0 || (!batchTesting && testingModelHashes.size > 0)} onClick={() => void (batchTesting ? cancelAllModelTests() : runModelTests(testTargets))}>{batchTesting ? t("取消全部测试") : t("一键测试")}</button>
      {selectedTargets.length > 0 && <button type="button" disabled={cursorBusy || batchTesting} onClick={() => void runModelTests(selectedTargets)}>{t("测试所选（{count}）", { count: selectedTargets.length })}</button>}
      {selectedModels.length > 0 && <button type="button" className={styles.deleteButton} disabled={cursorBusy || batchDeleting} onClick={() => setConfirmBatchDelete(true)}>{t("删除所选（{count}）", { count: selectedModels.length })}</button>}
    </div>
  </div>;

  const list = <div className={styles.list}>
    {listToolbarVisible && listToolbar}
    <CursorModelCards
      models={models}
      pluginModels={pluginModels}
      view={view}
      grouping={grouping}
      disabled={cursorBusy}
      testingModelHashes={testingModelHashes}
      testResults={modelTestResults}
      selectedModelHashes={selectedModelHashes}
      onToggleSelect={toggleModelSelection}
      onSelect={setModelSelection}
      onTest={(model) => void testModel(model)}
      onEdit={openEdit}
      onDuplicate={(model) => void duplicateModel(model)}
      onDelete={setDeleting}
      onTestPluginModel={(model) => void testModel({ model_hash: model.id, display_name: model.displayName })}
      onPluginSettings={() => navigate("/plugins")}
      onReorder={reorderModels}
      onGroupSettings={openGroupSettings}
    />
  </div>;

  const refreshCa = async () => {
    await appStore.refresh();
    if (appStore.getSnapshot().cursorHarness?.ca !== "ready") setWaitingForCaRefresh(false);
  };
  const openCaTerminal = () => {
    if (caCommand) void api.openCursorCaInstallTerminal(caCommand).catch((cause) => message(errorText(cause)));
    setCaCommand(null);
    setWaitingForCaRefresh(true);
  };
  const content = <CursorCaProvider><CursorCaGate busy={cursorBusy} waitingForRefresh={waitingForCaRefresh} onInitialize={() => void initializeCa()} onRefresh={() => void refreshCa()}>
    <div className={styles.page}>
      <LegacyModelImport>{({ busy: importingLegacyModels, previewing, open }) =>
        <CursorModelProvider><CursorModelGate busy={cursorBusy || importingLegacyModels} previewingImport={previewing} onAdd={openNew} onImport={open}>{list}</CursorModelGate></CursorModelProvider>
      }</LegacyModelImport>
    </div>
  </CursorCaGate></CursorCaProvider>;

  const editorTestState = editing ? modelTestResults.get(editing.model_hash) : undefined;
  const editorTesting = Boolean(editing && testingModelHashes.has(editing.model_hash));
  const activeGroups = grouping === "provider" ? providerGroups : typeGroups;
  const pluginSectionHeight = pluginModels.length > 0 ? 60 + pluginModels.length * 56 : 0;
  const listToolbarHeight = listToolbarVisible ? 44 : 0;
  const estimatedModelHeight = view === "grid"
    ? Math.max(380, Math.ceil(models.length / 3) * 196 + pluginSectionHeight + listToolbarHeight)
    : Math.max(380, (grouping === "none" ? 44 + models.length * 52 : activeGroups.reduce((height, group) => height + 44 + group.models.length * 52, 0) + Math.max(0, activeGroups.length - 1) * 20) + pluginSectionHeight + listToolbarHeight);

  return <>
    <PageActions>
      <div className={styles.takeoverActions}>
        <TooltipTrigger label={takeoverLabel}>
          <Switch
            checked={cursorTakenOver}
            disabled={cursorBusy || (!cursorTakenOver && !caReady)}
            label={takeoverLabel}
            onChange={(enabled) => {
              if (enabled) void appStore.setCursorEnabled(true);
              else setConfirmDisableTakeover(true);
            }}
          />
        </TooltipTrigger>
      </div>
      <button type="button" className={`${controls.secondary} ${controls.small}`} disabled={cursorBusy || !caReady} onClick={() => setBatchAddOpen(true)}>{t("批量添加")}</button>
      <TooltipTrigger label={caReady ? t("添加模型") : t("请先初始化 CA")}><button className={controls.iconButton} aria-label={t("添加模型")} disabled={!caReady || cursorBusy} onClick={openNew}><Icon icon={addIcon} size="1.1em" /></button></TooltipTrigger>
    </PageActions>
    <PageContent title="Cursor" sections={[{ key: "cursor-settings", estimatedHeight: estimatedModelHeight, content }]} />
    <ConfirmDialog
      open={confirmDisableTakeover}
      title={t("关闭接管Cursor？")}
      cancelLabel={t("取消")}
      confirmLabel={t("关闭接管")}
      onCancel={() => setConfirmDisableTakeover(false)}
      onConfirm={() => {
        setConfirmDisableTakeover(false);
        void appStore.setCursorEnabled(false);
      }}
    >
      <p>{t("关闭后将移除 Cursor 本地代理配置。如果你需要登陆官方账号，通常不需要关闭操作，推荐直接登陆你的账号即可(byok模型与官方账号的模型已支持无缝衔接)，是否继续关闭并清理代理？")}</p>
    </ConfirmDialog>
    <Modal fullHeight open={draft !== null} title={editing ? t("编辑模型") : t("添加模型")} banner={draft && (editorTesting || editorTestState) ? <CursorModelTestResult state={editorTestState} testing={editorTesting} /> : undefined} busy={cursorBusy || savingAndTesting} onClose={() => { if (editing && editorTesting) void cancelModelTest(editing.model_hash); setDraft(null); setEditing(null); }} onSubmit={() => void save()} submitLabel={t("保存")} secondaryAction={<button type="button" className={controls.secondary} disabled={cursorBusy || savingAndTesting} onClick={() => void (editorTesting && editing ? cancelModelTest(editing.model_hash) : saveAndTest())}>{savingAndTesting ? t("处理中…") : editorTesting ? t("取消测试") : t("保存并测试")}</button>}>
      {draft && <>
        <CursorModelEditor
          draft={draft}
          modelOptions={modelOptions}
          discovering={discovering}
          onChange={setDraft}
          onDiscover={discover}
        />
      </>}
    </Modal>
    {batchAddOpen && <CursorModelBatchAddModal
      models={models}
      busy={cursorBusy || batchAdding}
      onClose={() => setBatchAddOpen(false)}
      onAdd={(base, modelIds) => void addBatchModels(base, modelIds)}
    />}
    <ConfirmDialog open={caCommand !== null} title={t("安装本地 CA")} cancelLabel={t("关闭")} confirmLabel={t("打开终端")} onCancel={() => setCaCommand(null)} onConfirm={openCaTerminal}>
      <div className={styles.editor}><strong>{t("需要授权安装证书")}</strong><span>{t("安装命令已自动复制。点击“打开终端”，将命令粘贴到终端中执行，并按提示输入密码。")}</span><pre className={styles.command}>{caCommand}</pre></div>
    </ConfirmDialog>
    <Modal open={settingsGroup !== null} title={t("分组设置")} busy={groupSettingsBusy || cursorBusy} onClose={() => setSettingsGroup(null)} onSubmit={() => void saveGroupSettings()} submitLabel={t("保存")}>
      {settingsGroup && <div className={styles.editor}>
        <FormField label={t("分组名称")} hint={t("应用于该分组下的全部模型，并作为 Cursor 模型选择器中的徽章标签；清空则恢复显示服务器域名。")}>
          <TextInput placeholder={settingsGroup.key} value={groupNameDraft} onChange={(event) => setGroupNameDraft(event.target.value)} />
        </FormField>
        <FormField label={t("服务器地址")} hint={t("修改后应用于该分组下的全部模型；留空保持各模型现有配置不变。")}>
          <TextInput placeholder={t("留空保持不变")} value={groupBaseUrlDraft} onChange={(event) => setGroupBaseUrlDraft(event.target.value)} />
        </FormField>
        <FormField label="API Key" hint={t("修改后应用于该分组下的全部模型；留空保持各模型现有配置不变。")}>
          <SecretTextInput placeholder={t("留空保持不变")} autoComplete="off" value={groupApiKeyDraft} onChange={(event) => setGroupApiKeyDraft(event.target.value)} />
        </FormField>
      </div>}
    </Modal>
    <ConfirmDialog open={deleting !== null} title={t("删除模型")} cancelLabel={t("取消")} confirmLabel={t("删除")} onCancel={() => setDeleting(null)} onConfirm={() => { if (deleting) void appStore.deleteModel(deleting.model_hash); setDeleting(null); }}><p>{t("确定删除这个模型吗？")}</p></ConfirmDialog>
    <ConfirmDialog open={confirmBatchDelete} title={t("删除所选模型")} cancelLabel={t("取消")} confirmLabel={t("删除")} onCancel={() => setConfirmBatchDelete(false)} onConfirm={() => void deleteSelectedModels()}><p>{t("确定删除选中的 {count} 个模型吗？", { count: selectedModels.length })}</p></ConfirmDialog>
  </>;
}

/** 组内所有模型取值一致时返回该值,否则返回 null(表单留空表示保持不变)。 */
function sharedValue(values: string[]): string | null {
  const [first, ...rest] = values;
  if (first === undefined) return null;
  return rest.every((value) => value === first) ? first : null;
}

function errorText(cause: unknown) {
  return cause instanceof Error ? cause.message : String(cause);
}
