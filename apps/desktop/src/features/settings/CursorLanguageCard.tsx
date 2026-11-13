import { useCallback, useEffect, useState } from "react";
import { api, type CursorLanguageMode, type CursorLanguageStatus } from "../../shared/api";
import { Button } from "../../shared/ui/Button";
import { Select } from "../../shared/ui/Select";
import { Switch } from "../../shared/ui/Switch";
import { TitledCard } from "../../shared/ui/TitledCard";
import { useMessage } from "../../shared/ui/message";
import styles from "./CursorLanguageCard.module.scss";

/** Cursor 界面中文开关：默认开启，完整汉化与仅语言扩展两种模式可在设置里切换。 */
export function CursorLanguageCard() {
  const message = useMessage();
  const [status, setStatus] = useState<CursorLanguageStatus | null>(null);
  const [busy, setBusy] = useState(false);
  const [savingPath, setSavingPath] = useState(false);

  const report = useCallback((cause: unknown) => {
    message(cause instanceof Error ? cause.message : String(cause));
  }, [message]);

  const load = useCallback(() => {
    void api.cursorLanguage().then(setStatus).catch(report);
  }, [report]);

  useEffect(load, [load]);

  const apply = async (enabled: boolean, mode: CursorLanguageMode, notice: string) => {
    setBusy(true);
    try {
      setStatus(await api.setCursorLanguage({ enabled, mode }));
      message(notice);
    } catch (cause) {
      report(cause);
      load();
    } finally {
      setBusy(false);
    }
  };

  const savePath = async (path: string | null) => {
    setSavingPath(true);
    try {
      setStatus(await api.setCursorInstallPath(path));
      message(path ? t("已指定 Cursor 安装位置") : t("已恢复自动查找 Cursor"));
    } catch (cause) {
      report(cause);
      load();
    } finally {
      setSavingPath(false);
    }
  };

  const choosePath = async () => {
    try {
      const path = await api.pickDirectory(t("选择 Cursor 安装目录"));
      if (path) await savePath(path);
    } catch (cause) {
      report(cause);
    }
  };

  const stateLabel = (value: CursorLanguageStatus | null) => {
    if (!value) return t("读取中…");
    if (busy || value.state === "applying") return t("应用中…");
    switch (value.state) {
      case "applied": return t("已应用");
      case "pending": return t("待应用");
      case "degraded": return t("部分生效");
      case "unsupported": return t("未找到 Cursor");
      case "failed": return t("应用失败");
      default: return t("已关闭");
    }
  };

  const detail = status?.cursor_version
    ? t("Cursor {version} · 语言包 {pack}", { version: status.cursor_version, pack: status.pack_version })
    : null;

  return <TitledCard
    title={t("Cursor 中文界面")}
    action={<Button size="small" disabled={!status?.enabled || busy} onClick={() => {
      if (status) void apply(true, status.mode, t("已重新应用，请重启 Cursor"));
    }}>
      {t("重新应用")}
    </Button>}
  >
    <div className={styles.content}>
      <div className={styles.row}>
        <div className={styles.description}>
          <strong>{t("Cursor 界面语言")}</strong>
          <small>{t("开启后会安装社区中文语言包并设置 Cursor 显示语言；应用后需重启 Cursor。")}</small>
        </div>
        <Switch
          label={t("Cursor 界面语言")}
          checked={status?.enabled ?? true}
          disabled={!status || busy}
          onChange={(enabled) => void apply(
            enabled,
            status?.mode ?? "full",
            enabled ? t("Cursor 中文已开启，请重启 Cursor 查看效果") : t("Cursor 中文已关闭"),
          )}
        />
      </div>
      <div className={styles.row}>
        <div className={styles.description}>
          <strong>{t("汉化方式")}</strong>
          <small>{t("完整汉化会改写 Cursor 程序内的文案，需要管理员权限；仅语言扩展无需管理员权限，但部分界面仍是英文。")}</small>
        </div>
        <div className={styles.control}>
          <Select
            value={status?.mode ?? "full"}
            ariaLabel={t("汉化方式")}
            disabled={!status || busy || !status.enabled}
            options={[
              { value: "full", label: t("完整汉化") },
              { value: "extension", label: t("仅语言扩展") },
            ]}
            onChange={(mode) => void apply(
              status?.enabled ?? true,
              mode as CursorLanguageMode,
              t("汉化方式已切换，请重启 Cursor 查看效果"),
            )}
          />
        </div>
      </div>
      <div className={styles.row}>
        <div className={styles.description}>
          <strong>{t("Cursor 安装位置")}</strong>
          <small>{t("自动查找失败时可以手动选择 Cursor 安装目录；选择包含 resources/app 的文件夹即可。")}</small>
        </div>
        <div className={styles.pathControl}>
          <span className={styles.path} title={status?.install_path ?? status?.custom_path ?? undefined}>
            {status?.install_path ?? status?.custom_path ?? t("未找到 Cursor")}
          </span>
          <div className={styles.pathActions}>
            <Button size="small" disabled={!status || busy || savingPath} onClick={() => void choosePath()}>
              {t("选择目录")}
            </Button>
            {status?.custom_path && (
              <Button size="small" disabled={busy || savingPath} onClick={() => void savePath(null)}>
                {t("自动查找")}
              </Button>
            )}
          </div>
        </div>
      </div>
      <div className={styles.row}>
        <strong>{t("状态")}</strong>
        <span className={styles.value}>
          {stateLabel(status)}
          {detail && <small>{detail}</small>}
        </span>
      </div>
      {status?.error && <small className={styles.error}>{status.error}</small>}
    </div>
  </TitledCard>;
}
