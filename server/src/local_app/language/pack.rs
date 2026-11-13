//! Downloads the community Cursor language pack into the managed data directory.
use std::{fs, path::Path, time::Duration};

use crate::{Error, Result};

pub const PACKAGE: &str = "@polang233/cursor-language-pack";
/// 离线或查询失败时使用的版本；上游发布新版后会自动切换到最新版。
pub const DEFAULT_VERSION: &str = "1.1.4";

/// 语言包自带的安装脚本、字库和 VSIX；按包内布局落盘，脚本才能找到自己的资源。
const FILES: [&str; 10] = [
    "package.json",
    "config.json",
    "assets/language-pack.vsix",
    "scripts/translate-install.mjs",
    "scripts/lib/util.mjs",
    "scripts/lib/cursor-paths.mjs",
    "scripts/lib/commerce-i18n.mjs",
    "src/extension/main.cjs",
    "src/i18n/zh-cn/hardcoded.json",
    "src/i18n/zh-cn/commerce.json",
];

const SOURCES: [&str; 2] = [
    "https://unpkg.com",
    "https://cdn.jsdelivr.net/npm",
];

const MANIFESTS: [&str; 3] = [
    "https://registry.npmjs.org/@polang233/cursor-language-pack/latest",
    "https://unpkg.com/@polang233/cursor-language-pack/package.json",
    "https://cdn.jsdelivr.net/npm/@polang233/cursor-language-pack/package.json",
];

const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(120);
const VERSION_TIMEOUT: Duration = Duration::from_secs(20);

pub struct Pack {
    pub vsix: std::path::PathBuf,
}

/// 查询上游最新版本；网络或响应异常时返回 `None`，调用方继续使用已缓存的版本。
pub async fn latest_version() -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(VERSION_TIMEOUT)
        .build()
        .ok()?;
    for url in MANIFESTS {
        let response = match client.get(url).send().await {
            Ok(response) if response.status().is_success() => response,
            _ => continue,
        };
        let manifest: serde_json::Value = match response.json().await {
            Ok(manifest) => manifest,
            Err(_) => continue,
        };
        if let Some(version) = manifest.get("version").and_then(|value| value.as_str()) {
            if parse_version(version).is_some() {
                return Some(version.to_owned());
            }
        }
    }
    None
}

pub async fn ensure(root: &Path, version: &str) -> Result<Pack> {
    let missing = FILES
        .iter()
        .copied()
        .filter(|relative| !is_complete(root, relative))
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        let client = reqwest::Client::builder()
            .timeout(DOWNLOAD_TIMEOUT)
            .build()?;
        for relative in missing {
            download(&client, root, version, relative).await?;
        }
    }
    Ok(Pack {
        vsix: root.join("assets/language-pack.vsix"),
    })
}

/// 判断候选版本是否比当前版本新；版本号按数字段比较，解析失败时视为不更新。
pub fn is_newer(candidate: &str, current: &str) -> bool {
    match (parse_version(candidate), parse_version(current)) {
        (Some(candidate), Some(current)) => candidate > current,
        _ => false,
    }
}

fn parse_version(version: &str) -> Option<(u64, u64, u64)> {
    let mut parts = version.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    parts.next().is_none().then_some((major, minor, patch))
}

/// VSIX 是 zip 容器；体积过小或不以 `PK` 开头说明上次下载没有完成。
fn is_complete(root: &Path, relative: &str) -> bool {
    let path = root.join(relative);
    let Ok(metadata) = fs::metadata(&path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    if relative.ends_with(".vsix") {
        return metadata.len() > 1_000_000
            && fs::read(&path).is_ok_and(|bytes| bytes.starts_with(b"PK"));
    }
    metadata.len() > 0
}

async fn download(
    client: &reqwest::Client,
    root: &Path,
    version: &str,
    relative: &str,
) -> Result<()> {
    let mut failures = Vec::new();
    for source in SOURCES {
        let url = format!("{source}/{PACKAGE}@{version}/{relative}");
        match client.get(&url).send().await {
            Ok(response) if response.status().is_success() => match response.bytes().await {
                Ok(bytes) if !bytes.is_empty() => {
                    let target = root.join(relative);
                    if let Some(parent) = target.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    let temp = target.with_extension("part");
                    fs::write(&temp, &bytes)?;
                    fs::rename(&temp, &target)?;
                    return Ok(());
                }
                Ok(_) => failures.push(format!("{url}: 内容为空")),
                Err(error) => failures.push(format!("{url}: {error}")),
            },
            Ok(response) => failures.push(format!("{url}: HTTP {}", response.status())),
            Err(error) => failures.push(format!("{url}: {error}")),
        }
    }
    Err(Error::Config(format!(
        "下载 Cursor 语言包 {version} 失败（{}）。请检查网络后重试。",
        failures.join("；")
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newer_versions_are_compared_numerically() {
        assert!(is_newer("1.2.0", "1.1.4"));
        assert!(is_newer("1.10.0", "1.9.9"));
        assert!(!is_newer("1.1.4", "1.1.4"));
        assert!(!is_newer("1.0.9", "1.1.4"));
        assert!(!is_newer("latest", "1.1.4"));
    }
}
