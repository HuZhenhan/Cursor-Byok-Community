//! Locates the Cursor install and drives its command line, extensions and argv.json.
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use regex::Regex;
use serde_json::Value;
use tokio::{process::Command, time::timeout};

use crate::{Error, Result};

const COMMAND_TIMEOUT: Duration = Duration::from_secs(180);

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Clone, Debug)]
pub struct CursorInstall {
    pub root: PathBuf,
    pub app_root: PathBuf,
}

impl CursorInstall {
    pub fn version(&self) -> String {
        read_json(&self.app_root.join("product.json"))
            .or_else(|| read_json(&self.app_root.join("package.json")))
            .and_then(|manifest| manifest.get("version")?.as_str().map(str::to_owned))
            .unwrap_or_else(|| "unknown".into())
    }

    /// 完整汉化需要改写 `resources/app` 下的程序文件；系统级安装通常只有管理员能写。
    pub fn writable(&self) -> bool {
        let probe = self.app_root.join(".byok-write-test");
        if fs::write(&probe, b"").is_err() {
            return false;
        }
        let _ = fs::remove_file(probe);
        true
    }

    pub fn extensions_dir(&self) -> Result<PathBuf> {
        Ok(home_dir()?.join(self.data_folder()).join("extensions"))
    }

    pub fn argv_json(&self) -> Result<PathBuf> {
        Ok(home_dir()?.join(self.data_folder()).join("argv.json"))
    }

    fn data_folder(&self) -> String {
        read_json(&self.app_root.join("product.json"))
            .and_then(|product| product.get("dataFolderName")?.as_str().map(str::to_owned))
            .unwrap_or_else(|| ".cursor".into())
    }

    fn executable(&self) -> Result<PathBuf> {
        let names: &[&str] = if cfg!(windows) {
            &["Cursor.exe", "cursor.exe"]
        } else {
            &["Cursor", "cursor"]
        };
        let candidates = names
            .iter()
            .flat_map(|name| {
                [
                    self.root.join(name),
                    self.root.join("bin").join(name),
                    self.root.join("MacOS").join(name),
                ]
            })
            .find(|path| path.is_file());
        candidates.ok_or_else(|| Error::Config("未找到 Cursor 可执行文件".into()))
    }

    /// VS Code 的 Electron 主程序需要 `ELECTRON_RUN_AS_NODE` 才会按 CLI 运行，
    /// 而 Unix 上的 `bin/cursor` 本身就是命令行入口。
    fn cli(&self) -> Result<Command> {
        let executable = self.executable()?;
        let cli_js = self.app_root.join("out").join("cli.js");
        let direct = !cfg!(windows) && executable.ends_with(Path::new("bin").join("cursor"));
        let mut command = Command::new(&executable);
        if cli_js.is_file() && !direct {
            command.arg(cli_js);
            command.env("ELECTRON_RUN_AS_NODE", "1");
        }
        #[cfg(windows)]
        command.creation_flags(CREATE_NO_WINDOW);
        Ok(command)
    }
}

/// 优先使用用户手动指定的路径；未指定时按平台常见位置自动探测。
pub fn find_install(custom_path: Option<&str>) -> Option<CursorInstall> {
    if let Some(custom) = custom_path.map(str::trim).filter(|path| !path.is_empty()) {
        return install_from_path(Path::new(custom));
    }
    candidate_paths()
        .into_iter()
        .find_map(|path| install_from_path(&path))
}

/// 解析用户给出的安装位置：根目录、`resources/app`、macOS 的 `.app` 或 `Contents`
/// 目录，以及 Cursor 可执行文件本身都能识别。
pub fn install_from_path(path: &Path) -> Option<CursorInstall> {
    install_roots(path).into_iter().find_map(|root| {
        let app_root = root.join("resources").join("app");
        app_root
            .join("package.json")
            .is_file()
            .then(|| CursorInstall { root, app_root })
    })
}

fn install_roots(path: &Path) -> Vec<PathBuf> {
    if !path.exists() {
        return Vec::new();
    }
    let mut roots = Vec::new();
    if path.is_file() {
        // `which cursor` 在 Linux 上通常是 `/usr/bin/cursor` 之类的符号链接。
        if fs::symlink_metadata(path).is_ok_and(|meta| meta.file_type().is_symlink()) {
            if let Ok(real) = path.canonicalize() {
                roots.extend(install_roots(&real));
            }
        }
        if let Some(parent) = path.parent() {
            roots.push(parent.to_path_buf());
            // Unix 上的入口是 `bin/cursor`，macOS 是 `Contents/MacOS/Cursor`。
            if parent
                .file_name()
                .is_some_and(|name| name == "bin" || name == "MacOS")
            {
                roots.extend(parent.parent().map(Path::to_path_buf));
            }
        }
        return roots;
    }
    roots.push(path.to_path_buf());
    if path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("app"))
    {
        roots.push(path.join("Contents"));
    }
    // 用户选中 `resources` 或 `resources/app` 时逐级回到安装根目录。
    if let Some(parent) = path.parent() {
        roots.push(parent.to_path_buf());
        roots.extend(parent.parent().map(Path::to_path_buf));
    }
    roots
}

fn candidate_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(dir) = std::env::var_os("CURSOR_INSTALL_DIR") {
        paths.push(PathBuf::from(dir));
    }
    let home = dirs::home_dir().unwrap_or_default();
    if cfg!(windows) {
        #[cfg(windows)]
        paths.extend(registry_install_paths());
        let mut bases = Vec::new();
        if let Some(base) = std::env::var_os("LOCALAPPDATA") {
            bases.push(PathBuf::from(base).join("Programs"));
        }
        for key in ["ProgramFiles", "ProgramFiles(x86)"] {
            if let Some(base) = std::env::var_os(key) {
                bases.push(PathBuf::from(base));
            }
        }
        bases.push(home.join("AppData/Local/Programs"));
        for base in bases {
            paths.push(base.join("cursor"));
            paths.push(base.join("Cursor"));
        }
        paths.extend(executables_on_path(&["Cursor.exe", "cursor.exe"]));
    } else if cfg!(target_os = "macos") {
        paths.push(PathBuf::from("/Applications/Cursor.app"));
        paths.push(home.join("Applications/Cursor.app"));
    } else {
        for root in [
            "/usr/share/cursor",
            "/opt/Cursor",
            "/opt/cursor",
            "/usr/lib/cursor",
            "/snap/cursor/current/usr/share/cursor",
        ] {
            paths.push(PathBuf::from(root));
        }
        paths.push(home.join(".local/share/cursor"));
        paths.extend(executables_on_path(&["cursor", "Cursor"]));
    }
    paths
}

/// 便携版或通过包管理器安装时，可执行文件通常已经在 `PATH` 里。
fn executables_on_path(names: &[&str]) -> Vec<PathBuf> {
    let Some(path) = std::env::var_os("PATH") else {
        return Vec::new();
    };
    std::env::split_paths(&path)
        .flat_map(|dir| {
            names
                .iter()
                .map(move |name| dir.join(name))
                .collect::<Vec<_>>()
        })
        .filter(|candidate| candidate.is_file())
        .collect()
}

/// Cursor 注册了 `cursor://` 协议，注册表里的命令行记录了实际安装位置；
/// 这能覆盖安装到非默认目录的情况。
#[cfg(windows)]
fn registry_install_paths() -> Vec<PathBuf> {
    use windows_sys::Win32::System::Registry::{HKEY_CLASSES_ROOT, HKEY_CURRENT_USER};

    const PROTOCOL_KEY: &str = r"Software\Classes\cursor\shell\open\command";
    [HKEY_CURRENT_USER, HKEY_CLASSES_ROOT]
        .into_iter()
        .filter_map(|hive| read_registry_string(hive, PROTOCOL_KEY))
        .filter_map(|command| command_executable(&command).map(PathBuf::from))
        .filter(|path| path.is_file())
        .collect()
}

/// 从 `"C:\…\Cursor.exe" --open-url %1` 这类命令行中取出可执行文件路径。
fn command_executable(command: &str) -> Option<&str> {
    let command = command.trim();
    if let Some(rest) = command.strip_prefix('"') {
        return rest.find('"').map(|end| &rest[..end]);
    }
    let end = command
        .find(char::is_whitespace)
        .unwrap_or(command.len());
    (end > 0).then(|| &command[..end])
}

#[cfg(windows)]
fn read_registry_string(
    hive: windows_sys::Win32::System::Registry::HKEY,
    key: &str,
) -> Option<String> {
    use std::ptr;
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegQueryValueExW, KEY_READ, REG_SZ,
    };

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    let name = wide(key);
    let mut handle = ptr::null_mut();
    if unsafe { RegOpenKeyExW(hive, name.as_ptr(), 0, KEY_READ, &mut handle) } != 0 {
        return None;
    }
    let mut kind = 0;
    let mut size = 0u32;
    let mut value = None;
    let status = unsafe {
        RegQueryValueExW(
            handle,
            ptr::null(),
            ptr::null(),
            &mut kind,
            ptr::null_mut(),
            &mut size,
        )
    };
    if status == 0 && kind == REG_SZ && size > 0 {
        let mut buffer = vec![0u8; size as usize];
        let status = unsafe {
            RegQueryValueExW(
                handle,
                ptr::null(),
                ptr::null(),
                &mut kind,
                buffer.as_mut_ptr(),
                &mut size,
            )
        };
        if status == 0 {
            let text: Vec<u16> = buffer
                .chunks_exact(2)
                .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
                .collect();
            value = Some(
                String::from_utf16_lossy(&text)
                    .trim_end_matches('\0')
                    .to_owned(),
            );
        }
    }
    unsafe { RegCloseKey(handle) };
    value
}

/// 只检查扩展目录里是否已有指定版本的语言包，用于判断记录里的“已应用”是否仍然成立。
pub fn pack_installed(install: &CursorInstall, version: &str) -> bool {
    if version.is_empty() {
        return false;
    }
    let Ok(dir) = install.extensions_dir() else {
        return false;
    };
    let Ok(entries) = fs::read_dir(dir) else {
        return false;
    };
    let prefix = format!("polang233.cursor-language-pack-{version}");
    entries.flatten().any(|entry| {
        entry.file_name().to_string_lossy().starts_with(&prefix)
    })
}

pub fn read_locale(install: &CursorInstall) -> Option<String> {
    let text = fs::read_to_string(install.argv_json().ok()?).ok()?;
    let pattern = Regex::new(r#""locale"\s*:\s*"([^"]*)""#).ok()?;
    pattern
        .captures(&text)?
        .get(1)
        .map(|value| value.as_str().to_owned())
}

pub async fn install_extension(install: &CursorInstall, vsix: &Path) -> Result<()> {
    let mut command = install.cli()?;
    command.args([OsStr::new("--install-extension"), vsix.as_os_str()]);
    command.arg("--force");
    run(command, "安装 Cursor 语言扩展").await
}

pub async fn uninstall_extension(install: &CursorInstall, id: &str) -> Result<()> {
    let mut command = install.cli()?;
    command.args([OsStr::new("--uninstall-extension"), OsStr::new(id)]);
    run(command, "卸载 Cursor 语言扩展").await
}

/// 语言包占用 `vscode` 翻译 id，与官方 VS Code 中文包互斥；启用时移除冲突包。
pub async fn remove_conflicting_packs(install: &CursorInstall, keep: &str) {
    let Ok(dir) = install.extensions_dir() else {
        return;
    };
    for id in conflicting_packs(&dir, keep) {
        if let Err(error) = uninstall_extension(install, &id).await {
            tracing::warn!(%error, %id, "could not uninstall a conflicting Cursor language pack");
        }
    }
}

fn conflicting_packs(dir: &Path, keep: &str) -> Vec<String> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for entry in entries.flatten() {
        let manifest = read_json(&entry.path().join("package.json"));
        let Some(manifest) = manifest else {
            continue;
        };
        let id = format!(
            "{}.{}",
            manifest.get("publisher").and_then(Value::as_str).unwrap_or_default(),
            manifest.get("name").and_then(Value::as_str).unwrap_or_default()
        );
        if id.eq_ignore_ascii_case(keep) || !claims_vscode_zh_cn(&manifest) {
            continue;
        }
        found.push(id);
    }
    found
}

fn claims_vscode_zh_cn(manifest: &Value) -> bool {
    manifest
        .pointer("/contributes/localizations")
        .and_then(Value::as_array)
        .is_some_and(|localizations| {
            localizations.iter().any(|localization| {
                localization.get("languageId").and_then(Value::as_str) == Some("zh-cn")
                    && localization
                        .get("translations")
                        .and_then(Value::as_array)
                        .is_some_and(|translations| {
                            translations
                                .iter()
                                .any(|translation| translation.get("id").and_then(Value::as_str) == Some("vscode"))
                        })
            })
        })
}

/// 把语言包自带的安装脚本交给 Cursor 的 Electron 运行时执行（`ELECTRON_RUN_AS_NODE`），
/// 避免额外依赖系统 Node.js。
pub async fn run_pack_installer(install: &CursorInstall, script: &Path, vsix: &Path) -> Result<()> {
    let mut command = Command::new(install.executable()?);
    command.env("ELECTRON_RUN_AS_NODE", "1");
    command.arg(script);
    command.args([install_dir_flag(install), String::from("--locale=zh-cn")]);
    command.arg(vsix_flag(vsix));
    command.arg("--yes");
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);
    run(command, "改写 Cursor 程序文案").await
}

pub async fn run_pack_undo(install: &CursorInstall, script: &Path) -> Result<()> {
    let mut command = Command::new(install.executable()?);
    command.env("ELECTRON_RUN_AS_NODE", "1");
    command.arg(script);
    command.args([install_dir_flag(install), String::from("--undo")]);
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);
    run(command, "恢复 Cursor 程序文件").await
}

#[cfg(windows)]
pub async fn run_pack_installer_elevated(
    install: &CursorInstall,
    script: &Path,
    vsix: &Path,
    helper_dir: &Path,
) -> Result<()> {
    fs::create_dir_all(helper_dir)?;
    let helper = helper_dir.join("apply-language-pack.ps1");
    let inner = format!(
        "$env:ELECTRON_RUN_AS_NODE = '1'\r\n& '{}' '{}' '{}' '--locale=zh-cn' '{}' '--yes'\r\nexit $LASTEXITCODE\r\n",
        quote(install.executable()?.as_os_str()),
        quote(script.as_os_str()),
        quote(OsStr::new(&install_dir_flag(install))),
        quote(OsStr::new(&vsix_flag(vsix))),
    );
    fs::write(&helper, inner)?;
    elevate(&helper).await
}

#[cfg(windows)]
pub async fn run_pack_undo_elevated(
    install: &CursorInstall,
    script: &Path,
    helper_dir: &Path,
) -> Result<()> {
    fs::create_dir_all(helper_dir)?;
    let helper = helper_dir.join("undo-language-pack.ps1");
    let inner = format!(
        "$env:ELECTRON_RUN_AS_NODE = '1'\r\n& '{}' '{}' '{}' '--undo'\r\nexit $LASTEXITCODE\r\n",
        quote(install.executable()?.as_os_str()),
        quote(script.as_os_str()),
        quote(OsStr::new(&install_dir_flag(install))),
    );
    fs::write(&helper, inner)?;
    elevate(&helper).await
}

#[cfg(windows)]
async fn elevate(helper: &Path) -> Result<()> {
    let command_line = format!(
        "$p = Start-Process -FilePath 'powershell.exe' -ArgumentList @('-NoProfile','-ExecutionPolicy','Bypass','-File','{}') -Verb RunAs -Wait -PassThru; exit $p.ExitCode",
        quote(helper.as_os_str())
    );
    let powershell = std::env::var_os("SystemRoot")
        .map(|root| PathBuf::from(root).join("System32/WindowsPowerShell/v1.0/powershell.exe"))
        .filter(|path| path.is_file())
        .unwrap_or_else(|| PathBuf::from("powershell.exe"));
    let mut command = Command::new(powershell);
    command.args(["-NoProfile", "-NonInteractive", "-Command", command_line.as_str()]);
    command.creation_flags(CREATE_NO_WINDOW);
    run(command, "请求管理员权限").await
}

#[cfg(windows)]
fn quote(value: &OsStr) -> String {
    value.to_string_lossy().replace('\'', "''")
}

fn install_dir_flag(install: &CursorInstall) -> String {
    format!("--dir={}", install.root.display())
}

fn vsix_flag(vsix: &Path) -> String {
    format!("--vsix={}", vsix.display())
}

async fn run(mut command: Command, operation: &str) -> Result<()> {
    let output = timeout(COMMAND_TIMEOUT, command.output())
        .await
        .map_err(|_| Error::Config(format!("{operation}超时")))??;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let detail = if stderr.trim().is_empty() { stdout } else { stderr };
    Err(Error::Config(format!(
        "{operation}失败：{}",
        detail.trim().chars().take(800).collect::<String>()
    )))
}

/// 只改写 `locale` 字段，保留 argv.json 里的注释和排版。
pub fn write_locale(install: &CursorInstall, locale: &str) -> Result<()> {
    write_locale_at(&install.argv_json()?, locale)
}

fn write_locale_at(path: &Path, locale: &str) -> Result<()> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, format!("{{\n\t\"locale\": \"{locale}\"\n}}\n"))?;
            return Ok(());
        }
        Err(error) => return Err(error.into()),
    };
    let existing = Regex::new(r#"(?m)^([ \t]*)"locale"([ \t]*):([ \t]*)"[^"]*""#)
        .expect("locale field pattern");
    if existing.is_match(&text) {
        let updated = existing.replace(&text, format!("$1\"locale\"$2:$3\"{locale}\""));
        return fs::write(path, updated.as_bytes()).map_err(Into::into);
    }
    let close = text
        .rfind('}')
        .ok_or_else(|| Error::Config(format!("{} 不是 JSON 对象", path.display())))?;
    let head = text[..close].trim_end();
    let tail = &text[close..];
    let comments = Regex::new(r"//[^\n]*").expect("line comment pattern");
    let without_comments = comments.replace_all(head, "");
    let trimmed = without_comments.trim_end();
    let needs_comma = !trimmed.is_empty() && !trimmed.ends_with('{') && !trimmed.ends_with(',');
    // 行尾注释会吞掉同一行的逗号，这时把逗号放到下一行。
    let commented_tail = head.rsplit('\n').next().is_some_and(|line| line.contains("//"));
    let separator = if needs_comma && commented_tail {
        "\n,".to_owned()
    } else if needs_comma {
        ",".to_owned()
    } else {
        String::new()
    };
    fs::write(path, format!("{head}{separator}\n\t\"locale\": \"{locale}\"\n{tail}"))
        .map_err(Into::into)
}

fn read_json(path: &Path) -> Option<Value> {
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

fn home_dir() -> Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| Error::Config("cannot resolve user home directory".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_install() -> (tempfile::TempDir, CursorInstall) {
        let dir = tempfile::tempdir().unwrap();
        let app_root = dir.path().join("resources/app");
        fs::create_dir_all(&app_root).unwrap();
        fs::write(app_root.join("package.json"), "{}").unwrap();
        fs::write(
            app_root.join("product.json"),
            r#"{"version":"3.27.6","dataFolderName":".cursor"}"#,
        )
        .unwrap();
        let install = CursorInstall {
            root: dir.path().to_path_buf(),
            app_root,
        };
        (dir, install)
    }

    #[test]
    fn install_is_recognized_from_root_app_folder_and_executable() {
        let (dir, install) = temp_install();
        let root = dir.path();
        for path in [
            root.to_path_buf(),
            install.app_root.clone(),
            install.app_root.parent().unwrap().to_path_buf(),
        ] {
            let found = install_from_path(&path).expect("install should resolve");
            assert_eq!(found.app_root, install.app_root, "path: {}", path.display());
        }

        let executable = root.join(if cfg!(windows) { "Cursor.exe" } else { "Cursor" });
        fs::write(&executable, "").unwrap();
        assert_eq!(
            install_from_path(&executable).unwrap().app_root,
            install.app_root
        );

        assert!(install_from_path(&root.join("missing")).is_none());
    }

    #[test]
    fn registry_command_lines_resolve_the_executable() {
        assert_eq!(
            command_executable(r#""C:\Program Files\Cursor\Cursor.exe" --open-url %1"#),
            Some(r"C:\Program Files\Cursor\Cursor.exe")
        );
        assert_eq!(
            command_executable(r"C:\Cursor\Cursor.exe --open-url %1"),
            Some(r"C:\Cursor\Cursor.exe")
        );
        assert_eq!(command_executable("   "), None);
    }

    #[test]
    fn locale_is_inserted_without_touching_comments() {
        let (dir, _) = temp_install();
        let argv = dir.path().join("argv.json");
        fs::write(&argv, "{\n\t// keep me\n\t\"enable-crash-reporter\": true\n}\n").unwrap();

        write_locale_at(&argv, "zh-cn").unwrap();
        let text = fs::read_to_string(&argv).unwrap();
        assert!(text.contains("// keep me"));
        assert!(text.contains("\"locale\": \"zh-cn\""));
        assert!(text.contains("\"enable-crash-reporter\": true,"));
    }

    #[test]
    fn existing_locale_field_is_replaced_in_place() {
        let (dir, _) = temp_install();
        let argv = dir.path().join("argv.json");
        fs::write(&argv, "{\n\t\"locale\": \"zh-cn\"\n}\n").unwrap();

        write_locale_at(&argv, "en").unwrap();
        assert_eq!(
            fs::read_to_string(&argv).unwrap(),
            "{\n\t\"locale\": \"en\"\n}\n"
        );
    }
}
