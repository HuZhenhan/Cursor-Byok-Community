//! 在社区语言包之上补全 BYOK 维护的中文词条。
//!
//! 社区包按 Cursor 版本收录词库，新版界面仍会残留英文。BYOK 内置一份补充词条：
//! 完整汉化前并入安装脚本的硬编码词库，安装扩展后再覆盖扩展里的 NLS 翻译。
use std::{
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use serde_json::{json, Map, Value};

use crate::{Error, Result};

use super::cursor::CursorInstall;

/// 补充词库版本；词条变化时递增，让已应用的安装重新写入。
pub const VERSION: &str = "1";

const EXTENSION_PREFIX: &str = "polang233.cursor-language-pack-";
const MARKER: &str = ".byok-supplement.json";
const EMBEDDED: &str = include_str!("supplement.json");

fn supplement() -> &'static Value {
    static SUPPLEMENT: OnceLock<Value> = OnceLock::new();
    SUPPLEMENT.get_or_init(|| {
        serde_json::from_str(EMBEDDED).expect("supplement.json 由构建脚本生成，结构固定")
    })
}

fn section(name: &str) -> Option<&'static Map<String, Value>> {
    supplement().get(name)?.as_object()
}

/// 把补充的硬编码词条并入语言包自己的词库，供安装脚本改写 Cursor 程序文案。
pub fn merge_hardcoded(root: &Path) -> Result<()> {
    let Some(extra) = section("hardcoded") else {
        return Ok(());
    };
    let path = root.join("src/i18n/zh-cn/hardcoded.json");
    let mut dict = read_object(&path)?;
    let missing = extra
        .iter()
        .filter(|(source, _)| !dict.contains_key(*source))
        .count();
    if missing == 0 {
        return Ok(());
    }
    for (source, value) in extra {
        dict.entry(source.clone()).or_insert_with(|| value.clone());
    }
    write_json(&path, &Value::Object(dict))
}

/// 把补充的 NLS 词条写入指定版本的语言扩展；所有模块都落在 `main.i18n.json`。
pub fn apply_nls(install: &CursorInstall, version: &str) -> Result<()> {
    let Some(dir) = extension_dir(install, version) else {
        return Ok(());
    };
    apply_nls_in(&dir)
}

/// 指定版本的扩展里是否已有当前词库；没有安装扩展时无需补充。
pub fn is_applied(install: &CursorInstall, version: &str) -> bool {
    match extension_dir(install, version) {
        Some(dir) => is_applied_in(&dir),
        None => true,
    }
}

fn extension_dir(install: &CursorInstall, version: &str) -> Option<PathBuf> {
    let prefix = format!("{EXTENSION_PREFIX}{version}");
    let dir = install.extensions_dir().ok()?;
    fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .find(|path| {
            path.file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with(&prefix))
        })
}

fn apply_nls_in(dir: &Path) -> Result<()> {
    let path = dir.join("translations/zh-cn/main.i18n.json");
    let text = fs::read_to_string(&path).map_err(|error| {
        Error::Config(format!(
            "读取 Cursor 语言扩展翻译失败（{}）：{error}",
            path.display()
        ))
    })?;
    let mut document: Value = serde_json::from_str(&text)?;
    let contents = document
        .get_mut("contents")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| Error::Config("Cursor 语言扩展的翻译文件缺少 contents".into()))?;
    if let Some(nls) = section("nls") {
        for (module, entries) in nls {
            let Some(entries) = entries.as_object() else {
                continue;
            };
            let Some(target) = contents
                .entry(module.clone())
                .or_insert_with(|| Value::Object(Map::new()))
                .as_object_mut()
            else {
                continue;
            };
            for (key, value) in entries {
                target.insert(key.clone(), value.clone());
            }
        }
    }
    write_json(&path, &document)?;
    write_json(&dir.join(MARKER), &marker(&path))
}

/// 标记记录翻译文件的大小和修改时间；扩展被重装或升级后标记会失效，从而重新写入。
fn is_applied_in(dir: &Path) -> bool {
    let path = dir.join("translations/zh-cn/main.i18n.json");
    fs::read_to_string(dir.join(MARKER))
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .is_some_and(|value| value == marker(&path))
}

fn marker(path: &Path) -> Value {
    let count = |name: &str| {
        section(name)
            .map(|entries| entries.len())
            .unwrap_or_default()
    };
    let metadata = fs::metadata(path).ok();
    let bytes = metadata.as_ref().map(fs::Metadata::len).unwrap_or_default();
    let modified = metadata
        .and_then(|metadata| metadata.modified().ok())
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default();
    json!({
        "version": VERSION,
        "hardcoded": count("hardcoded"),
        "nls": count("nls"),
        "bytes": bytes,
        "modified": modified,
    })
}

fn read_object(path: &Path) -> Result<Map<String, Value>> {
    let text = fs::read_to_string(path).map_err(|error| {
        Error::Config(format!(
            "读取 Cursor 语言包词库失败（{}）：{error}",
            path.display()
        ))
    })?;
    match serde_json::from_str(&text)? {
        Value::Object(map) => Ok(map),
        _ => Err(Error::Config(format!("{} 不是 JSON 对象", path.display()))),
    }
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp = path.with_extension("byok-tmp");
    fs::write(&temp, serde_json::to_vec(value)?)?;
    fs::rename(&temp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn extension_with_main_i18n() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let translations = dir.path().join("translations/zh-cn");
        fs::create_dir_all(&translations).unwrap();
        fs::write(
            translations.join("main.i18n.json"),
            r#"{"version":"1","contents":{"keep":{},"existing":{"key":"保留"}}}"#,
        )
        .unwrap();
        dir
    }

    #[test]
    fn nls_entries_are_merged_without_dropping_existing_translations() {
        let dir = extension_with_main_i18n();
        assert!(!is_applied_in(dir.path()));

        apply_nls_in(dir.path()).unwrap();

        let document: Value = serde_json::from_str(
            &fs::read_to_string(dir.path().join("translations/zh-cn/main.i18n.json")).unwrap(),
        )
        .unwrap();
        let contents = document["contents"].as_object().unwrap();
        assert!(contents.contains_key("keep"));
        assert_eq!(contents["existing"]["key"], "保留");
        let changes = &contents["vs/sessions/contrib/changes/browser/changesView"];
        assert_eq!(changes["changesView.diffStats.files"], "{0} 个文件");
        assert!(is_applied_in(dir.path()));
    }
}
