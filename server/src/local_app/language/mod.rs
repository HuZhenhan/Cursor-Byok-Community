//! Applies the community Chinese language pack to the local Cursor install.
mod cursor;
mod pack;
mod supplement;

use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use crate::{
    config,
    local_app::process,
    store::{CursorLanguageApplied, CursorLanguageMode, CursorLanguageSettings, Store},
    Error, Result,
};

use self::cursor::CursorInstall;

const LOCALE: &str = "zh-cn";
const EXTENSION_ID: &str = "polang233.cursor-language-pack";
/// Cursor 进程检查间隔；退出后才会检查上游语言包并静默切换。
const WATCH_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CursorLanguageState {
    Disabled,
    Pending,
    Applying,
    Applied,
    Degraded,
    Unsupported,
    Failed,
}

#[derive(Clone, Debug, Serialize)]
pub struct CursorLanguageStatus {
    pub enabled: bool,
    pub mode: CursorLanguageMode,
    pub state: CursorLanguageState,
    pub cursor_version: Option<String>,
    pub pack_version: String,
    pub locale: &'static str,
    pub error: Option<String>,
    /// 当前实际使用的安装根目录；未找到 Cursor 时为空。
    pub install_path: Option<String>,
    /// 用户手动指定的安装位置；为空表示自动探测。
    pub custom_path: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct CursorLanguageInput {
    pub enabled: bool,
    pub mode: CursorLanguageMode,
}

/// 手动指定 Cursor 安装位置；`path` 为空时恢复自动探测。
#[derive(Clone, Debug, Deserialize)]
pub struct CursorInstallPathInput {
    pub path: Option<String>,
}

pub struct CursorLanguage {
    store: Store,
    /// 语言包缓存根目录，按版本分子目录存放。
    base: PathBuf,
    /// 当前选用的语言包版本；查询到上游新版后会在 Cursor 关闭时切换。
    version: RwLock<String>,
    applying: AtomicBool,
    error: RwLock<Option<String>>,
    apply_lock: tokio::sync::Mutex<()>,
}

impl CursorLanguage {
    pub fn new(store: Store) -> Result<Self> {
        Ok(Self {
            store,
            base: config::managed_data_dir()?.join("language-pack"),
            version: RwLock::new(pack::DEFAULT_VERSION.into()),
            applying: AtomicBool::new(false),
            error: RwLock::new(None),
            apply_lock: tokio::sync::Mutex::new(()),
        })
    }

    pub async fn status(&self) -> Result<CursorLanguageStatus> {
        let settings = self.store.cursor_language_settings().await?;
        let install = cursor::find_install(settings.install_path.as_deref());
        let cursor_version = install.as_ref().map(CursorInstall::version);
        let error = self.error.read().clone();
        let state = if !settings.enabled {
            CursorLanguageState::Disabled
        } else {
            match install.as_ref() {
                None => CursorLanguageState::Unsupported,
                Some(_) if self.applying.load(Ordering::Acquire) => CursorLanguageState::Applying,
                Some(install) if is_current(&settings, install) => {
                    if settings.applied.as_ref().map(|applied| applied.mode) == Some(settings.mode) {
                        CursorLanguageState::Applied
                    } else {
                        CursorLanguageState::Degraded
                    }
                }
                Some(_) if error.is_some() => CursorLanguageState::Failed,
                Some(_) => CursorLanguageState::Pending,
            }
        };
        let pack_version = settings
            .applied
            .as_ref()
            .map(|applied| applied.pack_version.clone())
            .filter(|version| !version.is_empty())
            .unwrap_or_else(|| self.version.read().clone());
        Ok(CursorLanguageStatus {
            enabled: settings.enabled,
            mode: settings.mode,
            state,
            cursor_version,
            pack_version,
            locale: LOCALE,
            error,
            install_path: install
                .as_ref()
                .map(|install| install.root.display().to_string()),
            custom_path: settings.install_path.clone(),
        })
    }

    /// 保存手动指定的安装位置；路径无效时返回错误，避免把错误配置写进状态。
    pub async fn set_install_path(
        &self,
        input: CursorInstallPathInput,
    ) -> Result<CursorLanguageStatus> {
        let mut settings = self.store.cursor_language_settings().await?;
        let path = input
            .path
            .map(|path| path.trim().to_owned())
            .filter(|path| !path.is_empty());
        if let Some(path) = path.as_deref() {
            if cursor::install_from_path(Path::new(path)).is_none() {
                return Err(Error::Config(
                    "所选目录不是 Cursor 安装目录，请选择包含 resources/app 的文件夹".into(),
                ));
            }
        }
        settings.install_path = path;
        self.store.set_cursor_language_settings(settings).await?;
        // 旧的失败提示只对旧的安装位置有意义。
        *self.error.write() = None;
        self.status().await
    }

    pub async fn update(&self, input: CursorLanguageInput) -> Result<CursorLanguageStatus> {
        let mut settings = self.store.cursor_language_settings().await?;
        settings.enabled = input.enabled;
        settings.mode = input.mode;
        self.store.set_cursor_language_settings(settings.clone()).await?;
        if settings.enabled {
            // 手动切换沿用已安装的版本，不在这里触发升级。
            let selected = installed_version(&settings, self.version.read().as_str());
            *self.version.write() = selected;
            self.apply(settings).await?;
        } else {
            self.disable(settings).await?;
        }
        self.status().await
    }

    /// 启动时按最近的设置自动应用；Cursor 正在运行时留给监视任务在退出后处理。
    pub async fn apply_on_startup(&self) {
        let settings = match self.store.cursor_language_settings().await {
            Ok(settings) => settings,
            Err(error) => {
                tracing::warn!(%error, "could not read Cursor language settings");
                return;
            }
        };
        if !settings.enabled
            || cursor::find_install(settings.install_path.as_deref()).is_none()
        {
            return;
        }
        if process::cursor_running().await.unwrap_or(true) {
            return;
        }
        if let Err(error) = self.refresh().await {
            tracing::warn!(%error, "applying the Cursor language pack on startup failed");
        }
    }

    /// 监视 Cursor 进程：从运行变为关闭时检查上游语言包，并在后台静默切换。
    pub async fn watch(&self, shutdown: CancellationToken) {
        let mut running = process::cursor_running().await.unwrap_or(true);
        let mut interval = tokio::time::interval(WATCH_INTERVAL);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tokio::select! {
                _ = shutdown.cancelled() => return,
                _ = interval.tick() => {}
            }
            let current = process::cursor_running().await.unwrap_or(true);
            if running && !current {
                if let Err(error) = self.refresh().await {
                    tracing::warn!(%error, "switching the Cursor language pack after Cursor exited failed");
                }
            }
            running = current;
        }
    }

    /// 查询上游最新版本并应用：只处理比已用版本更新的包，避免无谓改写。
    async fn refresh(&self) -> Result<()> {
        let settings = self.store.cursor_language_settings().await?;
        if !settings.enabled {
            return Ok(());
        }
        let Some(install) = cursor::find_install(settings.install_path.as_deref()) else {
            return Ok(());
        };
        // 系统级安装需要管理员权限，自动切换不弹提权窗口。
        if !install.writable() {
            return Ok(());
        }
        if let Some(latest) = pack::latest_version().await {
            let mut version = self.version.write();
            if pack::is_newer(&latest, version.as_str()) {
                *version = latest;
            }
        }
        let applied = settings.applied.as_ref();
        let applied_version = applied
            .map(|applied| applied.pack_version.as_str())
            .unwrap_or_default();
        let version = self.version.read().clone();
        // 离线启动时不要用默认版本覆盖已应用的更新版本。
        if pack::is_newer(applied_version, &version) {
            *self.version.write() = applied_version.to_owned();
        }
        let target = self.version.read().clone();
        let up_to_date = applied.is_some_and(|applied| applied.pack_version == target)
            && applied.map(|applied| applied.mode) == Some(settings.mode)
            && is_current(&settings, &install);
        if up_to_date {
            return Ok(());
        }
        tracing::info!(version = %target, "switching the Cursor language pack in the background");
        self.apply(settings).await
    }

    async fn apply(&self, mut settings: CursorLanguageSettings) -> Result<()> {
        let _guard = self.apply_lock.lock().await;
        let install = cursor::find_install(settings.install_path.as_deref())
            .ok_or_else(|| Error::Config("未找到 Cursor 安装目录".into()))?;
        let version = self.version.read().clone();
        self.applying.store(true, Ordering::Release);
        let result = self.apply_install(&install, settings.mode, &version).await;
        self.applying.store(false, Ordering::Release);
        match result {
            Ok((mode, warning)) => {
                settings.applied = Some(CursorLanguageApplied {
                    mode,
                    cursor_version: install.version(),
                    pack_version: version,
                    supplement_version: supplement::VERSION.into(),
                });
                self.store.set_cursor_language_settings(settings).await?;
                *self.error.write() = warning;
                Ok(())
            }
            Err(error) => {
                *self.error.write() = Some(error.to_string());
                Err(error)
            }
        }
    }

    async fn apply_install(
        &self,
        install: &CursorInstall,
        mode: CursorLanguageMode,
        version: &str,
    ) -> Result<(CursorLanguageMode, Option<String>)> {
        let root = self.base.join(version);
        let pack = pack::ensure(&root, version).await?;
        supplement::merge_hardcoded(&root)?;
        let script = root.join("scripts/translate-install.mjs");
        let result = match mode {
            CursorLanguageMode::Full => match self.apply_full(install, &pack, &script).await {
                Ok(()) => (CursorLanguageMode::Full, None),
                Err(error) => {
                    tracing::warn!(%error, "full Cursor language patch failed; falling back to the extension");
                    self.apply_extension(install, &pack).await?;
                    (
                        CursorLanguageMode::Extension,
                        Some(format!("完整汉化失败，已安装语言扩展：{error}")),
                    )
                }
            },
            CursorLanguageMode::Extension => {
                self.apply_extension(install, &pack).await?;
                (CursorLanguageMode::Extension, None)
            }
        };
        supplement::apply_nls(install, version)?;
        Ok(result)
    }

    async fn apply_full(
        &self,
        install: &CursorInstall,
        pack: &pack::Pack,
        script: &std::path::Path,
    ) -> Result<()> {
        if install.writable() {
            return cursor::run_pack_installer(install, script, &pack.vsix).await;
        }
        #[cfg(windows)]
        {
            cursor::run_pack_installer_elevated(install, script, &pack.vsix, &self.base).await
        }
        #[cfg(not(windows))]
        {
            let _ = pack;
            Err(Error::Config(
                "需要写入 Cursor 安装目录的权限，请以管理员身份运行本应用后重试".into(),
            ))
        }
    }

    async fn apply_extension(&self, install: &CursorInstall, pack: &pack::Pack) -> Result<()> {
        cursor::install_extension(install, &pack.vsix).await?;
        cursor::remove_conflicting_packs(install, EXTENSION_ID).await;
        cursor::write_locale(install, LOCALE)
    }

    async fn disable(&self, mut settings: CursorLanguageSettings) -> Result<()> {
        let _guard = self.apply_lock.lock().await;
        if let Some(install) = cursor::find_install(settings.install_path.as_deref()) {
            if settings
                .applied
                .as_ref()
                .is_some_and(|applied| applied.mode == CursorLanguageMode::Full)
            {
                // 用实际打过补丁的语言包版本还原，脚本才能找到对应的备份记录。
                let version = installed_version(&settings, self.version.read().as_str());
                let script = self.base
                    .join(version)
                    .join("scripts/translate-install.mjs");
                if script.is_file() {
                    if let Err(error) = self.undo(&install, &script).await {
                        tracing::warn!(%error, "restoring Cursor program files failed");
                    }
                }
            }
            if let Err(error) = cursor::uninstall_extension(&install, EXTENSION_ID).await {
                tracing::warn!(%error, "uninstalling the Cursor language pack failed");
            }
            if let Err(error) = cursor::write_locale(&install, "en") {
                tracing::warn!(%error, "resetting the Cursor display language failed");
            }
        }
        *self.error.write() = None;
        settings.applied = None;
        self.store.set_cursor_language_settings(settings).await
    }

    async fn undo(&self, install: &CursorInstall, script: &std::path::Path) -> Result<()> {
        if install.writable() {
            return cursor::run_pack_undo(install, script).await;
        }
        #[cfg(windows)]
        {
            cursor::run_pack_undo_elevated(install, script, &self.base).await
        }
        #[cfg(not(windows))]
        {
            Err(Error::Config(
                "需要写入 Cursor 安装目录的权限，请以管理员身份运行本应用后重试".into(),
            ))
        }
    }
}

fn is_current(settings: &CursorLanguageSettings, install: &CursorInstall) -> bool {
    let Some(applied) = settings.applied.as_ref() else {
        return false;
    };
    applied.cursor_version == install.version()
        && applied.supplement_version == supplement::VERSION
        && cursor::pack_installed(install, &applied.pack_version)
        && supplement::is_applied(install, &applied.pack_version)
        && cursor::read_locale(install).as_deref() == Some(LOCALE)
}

/// 已应用的版本优先；还没有应用记录时沿用当前选择。
fn installed_version(settings: &CursorLanguageSettings, selected: &str) -> String {
    settings
        .applied
        .as_ref()
        .map(|applied| applied.pack_version.clone())
        .filter(|version| !version.is_empty())
        .unwrap_or_else(|| selected.to_owned())
}
