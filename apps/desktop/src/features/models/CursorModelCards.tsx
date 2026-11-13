import type { IconifyIcon } from "@iconify/react/offline";
import { useEffect, useRef, useState, type ReactNode } from "react";
import Sortable from "sortablejs";
import type { Model, PluginModelDescriptor } from "../../shared/api";
import { ActionMenu, type ActionMenuItem } from "../../shared/ui/ActionMenu";
import { Card } from "../../shared/ui/Card";
import { Checkbox } from "../../shared/ui/Checkbox";
import { Icon } from "../../shared/ui/Icon";
import { chevronDownIcon, chevronRightIcon, claudeIcon, dragIcon, editIcon, flatColorOrganizationIcon, openAiIcon } from "../../shared/ui/icons";
import { CursorModelTestResult, type CursorModelTestState } from "./CursorModelTestResult";
import styles from "./CursorSettings.module.scss";

export type CursorModelView = "table" | "grid";
export type CursorModelGrouping = "none" | "provider" | "type";

export type CursorModelGroup = {
  key: string;
  label: string;
  icon: IconifyIcon;
  models: Model[];
};

type CursorModelCardsProps = {
  models: Model[];
  pluginModels: PluginModelDescriptor[];
  view: CursorModelView;
  grouping: CursorModelGrouping;
  disabled: boolean;
  testingModelHashes: Set<string>;
  testResults: Map<string, CursorModelTestState>;
  selectedModelHashes: Set<string>;
  onToggleSelect: (modelHash: string) => void;
  /** 批量勾选:全选和分组全选共用。 */
  onSelect: (modelHashes: string[], selected: boolean) => void;
  onTest: (model: Model) => void;
  onEdit: (model: Model) => void;
  onDuplicate: (model: Model) => void;
  onDelete: (model: Model) => void;
  onTestPluginModel: (model: PluginModelDescriptor) => void;
  onPluginSettings: (model: PluginModelDescriptor) => void;
  onReorder: (modelHashes: string[]) => void;
  onGroupSettings: (group: CursorModelGroup) => void;
};

type ModelGridProps = Omit<CursorModelCardsProps, "view" | "grouping" | "pluginModels" | "onTestPluginModel" | "onPluginSettings"> & {
  sortable: boolean;
};

export function cursorModelGroups(models: Model[], grouping: Exclude<CursorModelGrouping, "none">): CursorModelGroup[] {
  const groups = new Map<string, CursorModelGroup>();
  for (const model of models) {
    const descriptor = grouping === "provider" ? providerGroup(model) : typeGroup(model);
    const group = groups.get(descriptor.key);
    if (group) {
      group.models.push(model);
    } else {
      groups.set(descriptor.key, { ...descriptor, models: [model] });
    }
  }
  return [...groups.values()];
}

export function CursorModelCards(props: CursorModelCardsProps) {
  const pluginSections = pluginGroups(props.pluginModels).map((group) => <CollapsibleGroup
    key={`${props.view}:${props.grouping}:${group.pluginId}`}
    label={group.pluginName}
    iconSrc={group.icon}
    defaultOpen
  >
    {group.models.map((model) => <PluginModelRow
      key={model.id}
      model={model}
      disabled={props.disabled}
      testing={props.testingModelHashes.has(model.id)}
      result={props.testResults.get(model.id)}
      selected={props.selectedModelHashes.has(model.id)}
      onToggleSelect={() => props.onToggleSelect(model.id)}
      onTest={() => props.onTestPluginModel(model)}
      onSettings={() => props.onPluginSettings(model)}
    />)}
  </CollapsibleGroup>);

  return <div className={styles.modelGroups}>
    {props.view === "grid"
      ? <ModelGrid {...props} sortable />
      : props.grouping === "none"
        ? <Card className={styles.tableCard}><ModelTable {...props} sortable selectAll /></Card>
        : cursorModelGroups(props.models, props.grouping).map((group) => {
          const hashes = group.models.map((model) => model.model_hash);
          const state = selectionState(hashes, props.selectedModelHashes);
          return <CollapsibleGroup
            key={group.key}
            label={group.label}
            icon={group.icon}
            defaultOpen={false}
            selection={{
              checked: state.all,
              indeterminate: state.some,
              disabled: props.disabled,
              onChange: (checked) => props.onSelect(hashes, checked),
            }}
            onSettings={props.grouping === "provider" ? () => props.onGroupSettings(group) : undefined}
          >
            <ModelTable {...props} models={group.models} sortable={false} />
          </CollapsibleGroup>;
        })}
    {pluginSections}
  </div>;
}

/** 计算一组模型的整选/半选状态。 */
function selectionState(hashes: string[], selected: ReadonlySet<string>) {
  const count = hashes.filter((hash) => selected.has(hash)).length;
  return { all: hashes.length > 0 && count === hashes.length, some: count > 0 && count < hashes.length };
}

function pluginGroups(models: PluginModelDescriptor[]) {
  const groups: { pluginId: string; pluginName: string; icon: string; models: PluginModelDescriptor[] }[] = [];
  for (const model of models) {
    let group = groups.find((candidate) => candidate.pluginId === model.pluginId);
    if (!group) {
      group = { pluginId: model.pluginId, pluginName: model.pluginName, icon: model.icon, models: [] };
      groups.push(group);
    }
    group.models.push(model);
  }
  return groups;
}

function CollapsibleGroup({ label, icon, iconSrc, defaultOpen = true, selection, onSettings, children }: {
  label: string;
  icon?: IconifyIcon;
  iconSrc?: string;
  defaultOpen?: boolean;
  /** 分组全选复选框;省略时不显示(例如插件模型分组)。 */
  selection?: { checked: boolean; indeterminate: boolean; disabled: boolean; onChange: (checked: boolean) => void };
  onSettings?: () => void;
  children: ReactNode;
}) {
  const [open, setOpen] = useState(defaultOpen);
  return <Card className={styles.groupCard}>
    <div className={styles.groupHeader}>
      {selection && <Checkbox
        checked={selection.checked}
        indeterminate={selection.indeterminate}
        disabled={selection.disabled}
        ariaLabel={t("全选该分组")}
        onChange={selection.onChange}
      />}
      <button
        type="button"
        className={styles.groupToggle}
        aria-expanded={open}
        onClick={() => setOpen((current) => !current)}
      >
        {icon && <Icon icon={icon} size="1.1em" />}
        {iconSrc && <Icon src={iconSrc} size="1.1em" />}
        <span className={styles.groupLabel}>{label}</span>
      </button>
      {onSettings && <button type="button" className={styles.groupSettings} onClick={onSettings}>
        <Icon icon={editIcon} size="1em" />
        {t("分组设置")}
      </button>}
      <button
        type="button"
        className={styles.groupChevron}
        tabIndex={-1}
        aria-hidden="true"
        onClick={() => setOpen((current) => !current)}
      >
        <Icon icon={open ? chevronDownIcon : chevronRightIcon} size="1em" />
      </button>
    </div>
    {open && <div className={styles.modelList}>{children}</div>}
  </Card>;
}

function ModelTable({
  models,
  sortable: sortableEnabled,
  selectAll = false,
  disabled,
  testingModelHashes,
  testResults,
  selectedModelHashes,
  onToggleSelect,
  onSelect,
  onTest,
  onEdit,
  onDuplicate,
  onDelete,
  onReorder,
}: ModelGridProps & { selectAll?: boolean }) {
  const body = useRef<HTMLDivElement>(null);
  const sortable = useRef<Sortable | null>(null);
  const currentModels = useRef(models);
  const reorder = useRef(onReorder);
  currentModels.current = models;
  reorder.current = onReorder;

  useEffect(() => {
    if (!sortableEnabled || !body.current) return;
    sortable.current = Sortable.create(body.current, {
      animation: 160,
      dataIdAttr: "data-model-hash",
      draggable: `.${styles.tableRow}`,
      handle: `.${styles.tableSortHandle}`,
      ghostClass: styles.sortGhost,
      chosenClass: styles.sortChosen,
      dragClass: styles.sortDragging,
      forceFallback: true,
      fallbackOnBody: true,
      fallbackTolerance: 3,
      onEnd: (event) => {
        const oldIndex = event.oldDraggableIndex ?? event.oldIndex;
        const newIndex = event.newDraggableIndex ?? event.newIndex;
        if (typeof oldIndex !== "number" || typeof newIndex !== "number" || oldIndex === newIndex) {
          sortable.current?.sort(currentModels.current.map((model) => model.model_hash), false);
          return;
        }
        const reordered = currentModels.current.slice();
        const [moved] = reordered.splice(oldIndex, 1);
        if (!moved || newIndex < 0 || newIndex > reordered.length) {
          sortable.current?.sort(currentModels.current.map((model) => model.model_hash), false);
          return;
        }
        reordered.splice(newIndex, 0, moved);
        reorder.current(reordered.map((model) => model.model_hash));
      },
    });
    return () => {
      sortable.current?.destroy();
      sortable.current = null;
    };
  }, [sortableEnabled]);

  useEffect(() => {
    sortable.current?.option("disabled", disabled);
    sortable.current?.sort(models.map((model) => model.model_hash), false);
  }, [disabled, models]);

  const hashes = models.map((model) => model.model_hash);
  const headerSelection = selectionState(hashes, selectedModelHashes);

  return <div className={styles.tableScroll}>
    <div className={styles.tableHeader}>
      <span aria-hidden="true" />
      {selectAll
        ? <Checkbox checked={headerSelection.all} indeterminate={headerSelection.some} disabled={disabled} ariaLabel={t("全选")} onChange={(checked) => onSelect(hashes, checked)} />
        : <span aria-hidden="true" />}
      <span>{t("模型")}</span>
      <span>{t("供应商")}</span>
      <span>{t("类型")}</span>
      <span>{t("上下文")}</span>
      <span>{t("备注")}</span>
      <span>{t("测速")}</span>
      <span>{t("操作")}</span>
    </div>
    <div ref={body} className={styles.tableBody}>
      {models.map((model) => <div className={styles.tableRow} data-model-hash={model.model_hash} key={model.model_hash}>
        <span className={styles.tableDragCell}>
          {sortableEnabled && <button type="button" className={styles.tableSortHandle} disabled={disabled} aria-label={t("拖动排序")} title={t("拖动排序")} onClick={(event) => event.stopPropagation()}>
            <Icon icon={dragIcon} size="1em" />
          </button>}
        </span>
        <Checkbox checked={selectedModelHashes.has(model.model_hash)} disabled={disabled} ariaLabel={t("选择 {model}", { model: model.display_name })} onChange={() => onToggleSelect(model.model_hash)} />
        <div className={styles.modelRowName}>
          <span className={styles.modelRowNameText}>{model.display_name}</span>
          <span className={styles.modelRowModelId}>{model.model_id}</span>
        </div>
        <span className={styles.tableText} title={model.base_url}>{model.group_name?.trim() || providerDomain(model.base_url)}</span>
        <span className={styles.tableText}>{typeGroup(model).label}</span>
        <span className={styles.tableText}>{formatTokenCount(model.context_window_tokens)}</span>
        <span className={styles.tableText} title={model.tooltip_data}>{model.tooltip_data.trim() || "—"}</span>
        <div className={styles.tableSpeedCell}>
          <CursorModelTestResult compact state={testResults.get(model.model_hash)} testing={testingModelHashes.has(model.model_hash)} />
        </div>
        <div className={styles.modelCardActions}>
          <ActionMenu iconOnly label={t("更多")} disabled={disabled && !testingModelHashes.has(model.model_hash)} items={modelActionItems({ model, disabled, testing: testingModelHashes.has(model.model_hash), onTest, onEdit, onDuplicate, onDelete })} />
        </div>
      </div>)}
    </div>
  </div>;
}

/** 列表行与卡片共用的行内操作菜单项。 */
function modelActionItems({ model, disabled, testing, onTest, onEdit, onDuplicate, onDelete }: {
  model: Model;
  disabled: boolean;
  testing: boolean;
  onTest: (model: Model) => void;
  onEdit: (model: Model) => void;
  onDuplicate: (model: Model) => void;
  onDelete: (model: Model) => void;
}): ActionMenuItem[] {
  return [
    { id: "test", label: testing ? t("取消测试") : t("测试"), disabled: disabled && !testing, onSelect: () => onTest(model) },
    { id: "edit", label: t("编辑"), disabled, onSelect: () => onEdit(model) },
    { id: "duplicate", label: t("复制"), disabled, onSelect: () => onDuplicate(model) },
    { id: "delete", label: t("删除"), disabled, onSelect: () => onDelete(model) },
  ];
}

function formatTokenCount(tokens: number | null): string {
  if (tokens === null) return "—";
  if (tokens >= 1000000) return `${Number((tokens / 1000000).toFixed(1))}M`;
  if (tokens >= 1000) return `${Number((tokens / 1000).toFixed(0))}K`;
  return String(tokens);
}

function PluginModelRow({ model, disabled, testing, result, selected, onToggleSelect, onTest, onSettings }: {
  model: PluginModelDescriptor;
  disabled: boolean;
  testing: boolean;
  result: CursorModelTestState | undefined;
  selected: boolean;
  onToggleSelect: () => void;
  onTest: () => void;
  onSettings: () => void;
}) {
  return <div className={styles.modelRow}>
    <Checkbox checked={selected} disabled={disabled} ariaLabel={t("选择 {model}", { model: model.displayName })} onChange={onToggleSelect} />
    <div className={styles.modelRowName}>
      <span className={styles.modelRowNameText}>{model.displayName}</span>
      <span className={styles.modelRowModelId}>{model.modelId}</span>
    </div>
    <CursorModelTestResult compact state={result} testing={testing} />
    <div className={styles.modelCardActions}>
      <ActionMenu iconOnly label={t("更多")} disabled={disabled && !testing} items={[
        { id: "test", label: testing ? t("取消测试") : t("测试"), disabled: disabled && !testing, onSelect: onTest },
        { id: "settings", label: t("设置"), disabled, onSelect: onSettings },
      ]} />
    </div>
  </div>;
}

function ModelGrid({
  models,
  sortable: sortableEnabled,
  disabled,
  testingModelHashes,
  testResults,
  selectedModelHashes,
  onToggleSelect,
  onSelect,
  onTest,
  onEdit,
  onDuplicate,
  onDelete,
  onReorder,
}: ModelGridProps) {
  const grid = useRef<HTMLDivElement>(null);
  const sortable = useRef<Sortable | null>(null);
  const currentModels = useRef(models);
  const reorder = useRef(onReorder);
  currentModels.current = models;
  reorder.current = onReorder;

  useEffect(() => {
    if (!sortableEnabled || !grid.current) return;
    sortable.current = Sortable.create(grid.current, {
      animation: 160,
      dataIdAttr: "data-model-hash",
      draggable: `.${styles.modelCard}`,
      handle: `.${styles.sortHandle}`,
      ghostClass: styles.sortGhost,
      chosenClass: styles.sortChosen,
      dragClass: styles.sortDragging,
      forceFallback: true,
      fallbackOnBody: true,
      fallbackTolerance: 3,
      onEnd: (event) => {
        const oldIndex = event.oldDraggableIndex ?? event.oldIndex;
        const newIndex = event.newDraggableIndex ?? event.newIndex;
        if (typeof oldIndex !== "number"
          || typeof newIndex !== "number"
          || oldIndex === newIndex) {
          sortable.current?.sort(currentModels.current.map((model) => model.model_hash), false);
          return;
        }
        const reordered = currentModels.current.slice();
        const [moved] = reordered.splice(oldIndex, 1);
        if (!moved || newIndex < 0 || newIndex > reordered.length) {
          sortable.current?.sort(currentModels.current.map((model) => model.model_hash), false);
          return;
        }
        reordered.splice(newIndex, 0, moved);
        reorder.current(reordered.map((model) => model.model_hash));
      },
    });
    return () => {
      sortable.current?.destroy();
      sortable.current = null;
    };
  }, [sortableEnabled]);

  useEffect(() => {
    sortable.current?.option("disabled", disabled);
    sortable.current?.sort(models.map((model) => model.model_hash), false);
  }, [disabled, models]);

  const hashes = models.map((model) => model.model_hash);
  const state = selectionState(hashes, selectedModelHashes);

  return <div className={styles.gridSection}>
    {models.length > 0 && <div className={styles.gridToolbar}>
      <Checkbox checked={state.all} indeterminate={state.some} disabled={disabled} label={t("全选")} onChange={(checked) => onSelect(hashes, checked)} />
    </div>}
    <div ref={grid} className={styles.modelGrid}>
      {models.map((model) => {
        const result = testResults.get(model.model_hash);
        const testing = testingModelHashes.has(model.model_hash);
        return <Card className={styles.modelCard} data-model-hash={model.model_hash} key={model.model_hash}>
          {sortableEnabled && <button type="button" className={styles.sortHandle} disabled={disabled} aria-label={t("拖动排序")} title={t("拖动排序")} onClick={(event) => event.stopPropagation()}>
            <Icon icon={dragIcon} size="1.25em" />
          </button>}
          <div className={styles.modelCardContent}>
            <div className={styles.modelCardTop}>
              <div className={styles.modelCardName}>
                <span className={styles.modelCardNameText}>{model.display_name}</span>
                <span className={styles.modelCardModelId}>{model.model_id}</span>
              </div>
              <div className={styles.modelCardBadges}>
                <span className={styles.modelTypeBadge}>
                  <Icon icon={model.type === "anthropic" ? claudeIcon : openAiIcon} />
                  {model.type === "anthropic" ? "Anthropic" : "OpenAI"}
                </span>
                <Checkbox checked={selectedModelHashes.has(model.model_hash)} disabled={disabled} ariaLabel={t("选择 {model}", { model: model.display_name })} onChange={() => onToggleSelect(model.model_hash)} />
              </div>
            </div>
            <div className={styles.modelCardTest}>
              <CursorModelTestResult state={result} testing={testing} />
            </div>
            <div className={styles.modelCardActions}>
              <ActionMenu iconOnly label={t("更多")} disabled={disabled && !testing} items={modelActionItems({ model, disabled, testing, onTest, onEdit, onDuplicate, onDelete })} />
            </div>
          </div>
        </Card>;
      })}
    </div>
  </div>;
}

function providerGroup(model: Model) {
  const key = providerDomain(model.base_url);
  const label = model.group_name?.trim() || key;
  return { key, label, icon: flatColorOrganizationIcon };
}

function providerDomain(baseUrl: string) {
  const value = baseUrl.trim();
  try {
    return new URL(value).hostname.toLowerCase() || value;
  } catch {
    try {
      return new URL(`https://${value}`).hostname.toLowerCase() || value;
    } catch {
      return value;
    }
  }
}

function typeGroup(model: Model) {
  if (model.type === "anthropic") return { key: "anthropic", label: "Anthropic", icon: claudeIcon };
  if (model.openai_endpoint === "/v1/chat/completions") return { key: "openai-chat", label: "OpenAI Chat", icon: openAiIcon };
  return { key: "openai-responses", label: "OpenAI Responses", icon: openAiIcon };
}
