//! ISO catalog, local library scanner, streaming HTTPS fetcher with SHA-256 verification,
//! optional external Fido.ps1 Windows URL resolver (with local URL cache to avoid Sentinel
//! rate-limits), and VirtualBox optical-drive attachment.
//!
//! Pure MIT/Apache-2.0 Rust: never embeds GPLv3 Fido code; Fido is only invoked as an
//! optional external script if present on the host.

use crate::creds::with_timeout_secs;
use crate::vm::{down, gate, run_capture, valid_name, vboxmanage_candidates};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsoCatalogEntry {
    pub id: String,
    pub name: String,
    pub category: String,
    pub arch: String,
    pub version: String,
    pub size_hint_mb: u32,
    pub vbox_ostype: String,
    pub download_url: Option<String>,
    pub checksum_url: Option<String>,
    pub provider: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalIsoItem {
    pub filename: String,
    pub path: String,
    pub size_bytes: u64,
    pub size_mb: f64,
    pub modified_utc: String,
    pub sha256_sidecar: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IsoFetchParams {
    pub id: Option<String>,
    pub url: Option<String>,
    pub dest_dir: Option<String>,
    pub fido_script: Option<String>,
    pub win_version: Option<String>,
    pub lang: Option<String>,
    pub arch: Option<String>,
    pub url_only: bool,
    pub confirm: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FidoCacheEntry {
    url: String,
    fetched_at_utc: String,
}

/// Curated Linux, UEFI Shell, and external-Fido Windows ISO catalog.
pub fn builtin_catalog() -> Vec<IsoCatalogEntry> {
    vec![
        IsoCatalogEntry {
            id: "alpine-3.21-virt".to_string(),
            name: "Alpine Linux 3.21 Virtual".to_string(),
            category: "linux".to_string(),
            arch: "x86_64".to_string(),
            version: "3.21.3".to_string(),
            size_hint_mb: 62,
            vbox_ostype: "Linux_64".to_string(),
            download_url: Some(
                "https://dl-cdn.alpinelinux.org/alpine/v3.21/releases/x86_64/alpine-virt-3.21.3-x86_64.iso"
                    .to_string(),
            ),
            checksum_url: Some(
                "https://dl-cdn.alpinelinux.org/alpine/v3.21/releases/x86_64/alpine-virt-3.21.3-x86_64.iso.sha256"
                    .to_string(),
            ),
            provider: "direct".to_string(),
            notes: "Lightweight headless VM kernel (~62 MB); fast smoke-test guest.".to_string(),
        },
        IsoCatalogEntry {
            id: "ubuntu-24.04-server".to_string(),
            name: "Ubuntu 24.04.2 LTS Server".to_string(),
            category: "linux".to_string(),
            arch: "amd64".to_string(),
            version: "24.04.2".to_string(),
            size_hint_mb: 2650,
            vbox_ostype: "Ubuntu_64".to_string(),
            download_url: Some(
                "https://releases.ubuntu.com/24.04.2/ubuntu-24.04.2-live-server-amd64.iso"
                    .to_string(),
            ),
            checksum_url: Some("https://releases.ubuntu.com/24.04.2/SHA256SUMS".to_string()),
            provider: "direct".to_string(),
            notes: "Official LTS server ISO; ideal for clean-room Linux craft testing.".to_string(),
        },
        IsoCatalogEntry {
            id: "ubuntu-24.04-desktop".to_string(),
            name: "Ubuntu 24.04.2 LTS Desktop".to_string(),
            category: "linux".to_string(),
            arch: "amd64".to_string(),
            version: "24.04.2".to_string(),
            size_hint_mb: 6050,
            vbox_ostype: "Ubuntu_64".to_string(),
            download_url: Some(
                "https://releases.ubuntu.com/24.04.2/ubuntu-24.04.2-desktop-amd64.iso".to_string(),
            ),
            checksum_url: Some("https://releases.ubuntu.com/24.04.2/SHA256SUMS".to_string()),
            provider: "direct".to_string(),
            notes: "Full GNOME desktop LTS image.".to_string(),
        },
        IsoCatalogEntry {
            id: "debian-12-netinst".to_string(),
            name: "Debian 12 Bookworm Netinst".to_string(),
            category: "linux".to_string(),
            arch: "amd64".to_string(),
            version: "12.10.0".to_string(),
            size_hint_mb: 660,
            vbox_ostype: "Debian_64".to_string(),
            download_url: Some(
                "https://cdimage.debian.org/debian-cd/current/amd64/iso-cd/debian-12.10.0-amd64-netinst.iso"
                    .to_string(),
            ),
            checksum_url: Some(
                "https://cdimage.debian.org/debian-cd/current/amd64/iso-cd/SHA256SUMS".to_string(),
            ),
            provider: "direct".to_string(),
            notes: "Minimal Debian network installer ISO.".to_string(),
        },
        IsoCatalogEntry {
            id: "fedora-41-workstation".to_string(),
            name: "Fedora 41 Workstation Live".to_string(),
            category: "linux".to_string(),
            arch: "x86_64".to_string(),
            version: "41-1.4".to_string(),
            size_hint_mb: 2450,
            vbox_ostype: "Fedora_64".to_string(),
            download_url: Some(
                "https://download.fedoraproject.org/pub/fedora/linux/releases/41/Workstation/x86_64/iso/Fedora-Workstation-Live-x86_64-41-1.4.iso"
                    .to_string(),
            ),
            checksum_url: Some(
                "https://download.fedoraproject.org/pub/fedora/linux/releases/41/Workstation/x86_64/iso/Fedora-Workstation-41-1.4-x86_64-CHECKSUM"
                    .to_string(),
            ),
            provider: "direct".to_string(),
            notes: "Modern Fedora desktop live image.".to_string(),
        },
        IsoCatalogEntry {
            id: "archlinux-latest".to_string(),
            name: "Arch Linux Latest".to_string(),
            category: "linux".to_string(),
            arch: "x86_64".to_string(),
            version: "latest".to_string(),
            size_hint_mb: 1150,
            vbox_ostype: "ArchLinux_64".to_string(),
            download_url: Some(
                "https://geo.mirror.pkgbuild.com/iso/latest/archlinux-x86_64.iso".to_string(),
            ),
            checksum_url: Some(
                "https://geo.mirror.pkgbuild.com/iso/latest/sha256sums.txt".to_string(),
            ),
            provider: "direct".to_string(),
            notes: "Rolling monthly Arch live ISO via GeoIP mirror.".to_string(),
        },
        IsoCatalogEntry {
            id: "uefi-shell-2.2-release".to_string(),
            name: "UEFI Shell 2.2 (Release)".to_string(),
            category: "uefi".to_string(),
            arch: "x64/aa64/ia32".to_string(),
            version: "24H2".to_string(),
            size_hint_mb: 3,
            vbox_ostype: "Other_64".to_string(),
            download_url: Some(
                "https://github.com/pbatard/UEFI-Shell/releases/download/24H2/UEFI-Shell-2.2-24H2-RELEASE.iso"
                    .to_string(),
            ),
            checksum_url: None,
            provider: "direct".to_string(),
            notes: "Bootable EDK2 UEFI 2.2 Shell ISO (~3 MB) for firmware/boot diagnostics."
                .to_string(),
        },
        IsoCatalogEntry {
            id: "uefi-shell-2.2-debug".to_string(),
            name: "UEFI Shell 2.2 (Debug)".to_string(),
            category: "uefi".to_string(),
            arch: "x64/aa64/ia32".to_string(),
            version: "24H2".to_string(),
            size_hint_mb: 5,
            vbox_ostype: "Other_64".to_string(),
            download_url: Some(
                "https://github.com/pbatard/UEFI-Shell/releases/download/24H2/UEFI-Shell-2.2-24H2-DEBUG.iso"
                    .to_string(),
            ),
            checksum_url: None,
            provider: "direct".to_string(),
            notes: "Bootable EDK2 UEFI 2.2 Shell Debug build with verbose assertions.".to_string(),
        },
        IsoCatalogEntry {
            id: "windows-11-latest".to_string(),
            name: "Windows 11 Latest (via Fido.ps1)".to_string(),
            category: "windows".to_string(),
            arch: "x64".to_string(),
            version: "Latest".to_string(),
            size_hint_mb: 5800,
            vbox_ostype: "Windows11_64".to_string(),
            download_url: None,
            checksum_url: None,
            provider: "fido-external".to_string(),
            notes: "Resolves signed Microsoft CDN link via external Fido.ps1 (cached 20h to avoid Sentinel rate-limits)."
                .to_string(),
        },
        IsoCatalogEntry {
            id: "windows-10-latest".to_string(),
            name: "Windows 10 Latest (via Fido.ps1)".to_string(),
            category: "windows".to_string(),
            arch: "x64".to_string(),
            version: "Latest".to_string(),
            size_hint_mb: 5700,
            vbox_ostype: "Windows10_64".to_string(),
            download_url: None,
            checksum_url: None,
            provider: "fido-external".to_string(),
            notes: "Resolves signed Microsoft CDN link via external Fido.ps1 (cached 20h)."
                .to_string(),
        },
    ]
}

/// Resolve the default local ISO cache directory (`VMCRAFT_ISO_DIR` or `~/.vmcraft/isos`).
pub fn default_iso_dir() -> PathBuf {
    if let Ok(custom) = std::env::var("VMCRAFT_ISO_DIR") {
        let trimmed = custom.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
        let trimmed = home.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed).join(".vmcraft").join("isos");
        }
    }
    PathBuf::from(".vmcraft").join("isos")
}

/// Locate an external `Fido.ps1` script on the host (never bundled).
pub fn find_fido_script(explicit: Option<&str>) -> Option<PathBuf> {
    if let Some(p) = explicit {
        let trimmed = p.trim();
        if !trimmed.is_empty() {
            let pb = PathBuf::from(trimmed);
            if pb.is_file() {
                return Some(pb);
            }
        }
    }
    if let Ok(env_p) = std::env::var("VMCRAFT_FIDO_SCRIPT") {
        let trimmed = env_p.trim();
        if !trimmed.is_empty() {
            let pb = PathBuf::from(trimmed);
            if pb.is_file() {
                return Some(pb);
            }
        }
    }
    let default_candidate = default_iso_dir().join("Fido.ps1");
    if default_candidate.is_file() {
        return Some(default_candidate);
    }
    None
}

pub fn iso_catalog(category: Option<&str>) -> Value {
    let filter = category
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty() && s != "all");
    let entries: Vec<IsoCatalogEntry> = builtin_catalog()
        .into_iter()
        .filter(|e| match &filter {
            Some(cat) => e.category == *cat,
            None => true,
        })
        .collect();
    let fido_path = find_fido_script(None);
    json!({
        "tool": "iso.catalog",
        "status": "ok",
        "category": filter.unwrap_or_else(|| "all".to_string()),
        "count": entries.len(),
        "default_iso_dir": default_iso_dir().display().to_string(),
        "fido_detected": fido_path.as_ref().map(|p| p.display().to_string()),
        "entries": entries,
    })
}

pub fn iso_list(dir: Option<&str>) -> Value {
    let target_dir = dir
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(default_iso_dir);

    let mut items: Vec<LocalIsoItem> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&target_dir) {
        for entry in rd.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_ascii_lowercase())
                .unwrap_or_default();
            if ext != "iso" && ext != "img" {
                continue;
            }
            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            let size_bytes = meta.len();
            let modified_utc = meta
                .modified()
                .ok()
                .map(|t| {
                    let dt: DateTime<Utc> = t.into();
                    dt.to_rfc3339()
                })
                .unwrap_or_default();
            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown.iso")
                .to_string();
            let sidecar_path = PathBuf::from(format!("{}.sha256", path.display()));
            let sha256_sidecar = std::fs::read_to_string(&sidecar_path)
                .ok()
                .and_then(|s| s.split_whitespace().next().map(|h| h.to_ascii_lowercase()))
                .filter(|h| h.len() == 64 && h.chars().all(|c| c.is_ascii_hexdigit()));

            items.push(LocalIsoItem {
                filename,
                path: path.display().to_string(),
                size_bytes,
                size_mb: (size_bytes as f64 / (1024.0 * 1024.0) * 10.0).round() / 10.0,
                modified_utc,
                sha256_sidecar,
            });
        }
    }
    items.sort_by_key(|a| a.filename.to_lowercase());
    let fido_path = find_fido_script(None);

    json!({
        "tool": "iso.list",
        "status": "ok",
        "dir": target_dir.display().to_string(),
        "exists": target_dir.is_dir(),
        "count": items.len(),
        "fido_detected": fido_path.as_ref().map(|p| p.display().to_string()),
        "isos": items,
    })
}

pub async fn iso_verify(path: &str, expected_sha256: Option<&str>) -> Value {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return json!({"tool": "iso.verify", "status": "error", "reason": "path is required"});
    }
    let file_path = Path::new(trimmed);
    let mut file = match tokio::fs::File::open(file_path).await {
        Ok(f) => f,
        Err(e) => {
            return json!({
                "tool": "iso.verify",
                "status": "error",
                "path": trimmed,
                "reason": format!("open failed: {e}")
            });
        }
    };

    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1024 * 1024];
    let mut total_bytes: u64 = 0;
    loop {
        match file.read(&mut buf).await {
            Ok(0) => break,
            Ok(n) => {
                hasher.update(&buf[..n]);
                total_bytes += n as u64;
            }
            Err(e) => {
                return json!({
                    "tool": "iso.verify",
                    "status": "error",
                    "path": trimmed,
                    "reason": format!("read failed: {e}")
                });
            }
        }
    }
    let actual = format!("{:x}", hasher.finalize());

    let expected = expected_sha256
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            let sidecar = PathBuf::from(format!("{trimmed}.sha256"));
            std::fs::read_to_string(sidecar)
                .ok()
                .and_then(|s| s.split_whitespace().next().map(|h| h.to_ascii_lowercase()))
        });

    let verified = expected.as_ref().map(|exp| exp == &actual);
    let status = match verified {
        Some(false) => "error",
        _ => "ok",
    };

    json!({
        "tool": "iso.verify",
        "status": status,
        "path": trimmed,
        "size_bytes": total_bytes,
        "sha256": actual,
        "expected_sha256": expected,
        "verified": verified,
    })
}

fn validate_https_url(url: &str) -> Result<(), String> {
    let trimmed = url.trim();
    if !trimmed.starts_with("https://") {
        return Err("only https:// URLs are permitted for ISO downloads".to_string());
    }
    if trimmed.len() > 2048 || trimmed.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err("invalid URL characters or length".to_string());
    }
    Ok(())
}

pub(crate) fn sanitize_iso_filename(url: &str, fallback_id: Option<&str>) -> String {
    let path_part = url.split('?').next().unwrap_or(url);
    let candidate = path_part.rsplit('/').next().unwrap_or("");
    let cleaned: String = candidate
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
        .collect();
    if !cleaned.is_empty()
        && !cleaned.starts_with('.')
        && (cleaned.to_ascii_lowercase().ends_with(".iso")
            || cleaned.to_ascii_lowercase().ends_with(".img"))
    {
        return cleaned;
    }
    let stem: String = fallback_id
        .unwrap_or("download")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
        .collect();
    let safe_stem = if stem.is_empty() {
        "download".to_string()
    } else {
        stem
    };
    format!("{safe_stem}.iso")
}

fn fido_cache_path(iso_dir: &Path) -> PathBuf {
    iso_dir.join(".fido-url-cache.json")
}

fn load_fido_cache_hit(iso_dir: &Path, key: &str, now: DateTime<Utc>) -> Option<String> {
    let raw = std::fs::read_to_string(fido_cache_path(iso_dir)).ok()?;
    let map: HashMap<String, FidoCacheEntry> = serde_json::from_str(&raw).ok()?;
    let entry = map.get(key)?;
    let fetched = DateTime::parse_from_rfc3339(&entry.fetched_at_utc)
        .ok()?
        .with_timezone(&Utc);
    if now.signed_duration_since(fetched) < ChronoDuration::hours(20) {
        Some(entry.url.clone())
    } else {
        None
    }
}

fn save_fido_cache_hit(iso_dir: &Path, key: &str, url: &str, now: DateTime<Utc>) {
    let path = fido_cache_path(iso_dir);
    let mut map: HashMap<String, FidoCacheEntry> = std::fs::read_to_string(&path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default();
    map.insert(
        key.to_string(),
        FidoCacheEntry {
            url: url.to_string(),
            fetched_at_utc: now.to_rfc3339(),
        },
    );
    if let Ok(json_str) = serde_json::to_string_pretty(&map) {
        let _ = std::fs::write(path, json_str);
    }
}

async fn resolve_fido_url(
    fido_path: &Path,
    win_version: &str,
    lang: &str,
    arch: &str,
) -> Result<String, String> {
    let pwsh = if cfg!(windows) {
        let pwsh7 = r"C:\Program Files\PowerShell\7\pwsh.exe";
        if Path::new(pwsh7).is_file() {
            pwsh7.to_string()
        } else {
            "powershell.exe".to_string()
        }
    } else {
        "pwsh".to_string()
    };

    let fido_str = fido_path.display().to_string();
    let mut args = vec![
        "-NoProfile",
        "-ExecutionPolicy",
        "Bypass",
        "-File",
        &fido_str,
        "-Win",
        win_version,
        "-Rel",
        "Latest",
        "-Arch",
        arch,
        "-GetUrl",
    ];
    if !lang.trim().is_empty() && !lang.eq_ignore_ascii_case("default") {
        args.push("-Lang");
        args.push(lang);
    }

    let out = run_capture(&pwsh, &args, 45).await?;
    let url = out
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("https://"))
        .ok_or_else(|| format!("Fido.ps1 did not return an https:// URL: {}", out.trim()))?;
    validate_https_url(url)?;
    Ok(url.to_string())
}

pub async fn iso_fetch(params: IsoFetchParams) -> Value {
    let dest_dir = params
        .dest_dir
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(default_iso_dir);

    let catalog = builtin_catalog();
    let entry = params
        .id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .and_then(|id| catalog.iter().find(|e| e.id == id));

    if let Some(requested_id) = params.id.as_deref().filter(|s| !s.trim().is_empty()) {
        if entry.is_none() && params.url.as_deref().unwrap_or("").trim().is_empty() {
            return json!({
                "tool": "iso.fetch",
                "status": "error",
                "reason": format!("unknown catalog id '{requested_id}'; run iso.catalog to list valid IDs")
            });
        }
    }

    let mut cached_url_hit = false;
    let resolved_url = if let Some(u) = params
        .url
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        if let Err(e) = validate_https_url(u) {
            return json!({"tool": "iso.fetch", "status": "error", "reason": e});
        }
        u.to_string()
    } else if let Some(cat_entry) = entry {
        if cat_entry.provider == "fido-external" {
            let win_ver = params
                .win_version
                .as_deref()
                .unwrap_or(if cat_entry.id.contains("10") {
                    "Windows 10"
                } else {
                    "Windows 11"
                });
            let lang = params.lang.as_deref().unwrap_or("default");
            let arch = params.arch.as_deref().unwrap_or("x64");
            let cache_key = format!("{win_ver}|{lang}|{arch}");
            if let Some(hit) = load_fido_cache_hit(&dest_dir, &cache_key, Utc::now()) {
                cached_url_hit = true;
                hit
            } else {
                let fido_path = match find_fido_script(params.fido_script.as_deref()) {
                    Some(p) => p,
                    None => {
                        return json!({
                            "tool": "iso.fetch",
                            "status": "down",
                            "reason": "Windows ISO resolution requires external Fido.ps1 (GPLv3, not bundled). Place Fido.ps1 in ~/.vmcraft/isos/ or set VMCRAFT_FIDO_SCRIPT / --fido-script.",
                            "fido_repo": "https://github.com/pbatard/Fido",
                            "expected_dir": dest_dir.display().to_string(),
                        });
                    }
                };
                match resolve_fido_url(&fido_path, win_ver, lang, arch).await {
                    Ok(u) => {
                        let _ = std::fs::create_dir_all(&dest_dir);
                        save_fido_cache_hit(&dest_dir, &cache_key, &u, Utc::now());
                        u
                    }
                    Err(e) => {
                        return json!({
                            "tool": "iso.fetch",
                            "status": "error",
                            "reason": format!("Fido.ps1 resolution failed: {e}")
                        });
                    }
                }
            }
        } else {
            match &cat_entry.download_url {
                Some(u) => u.clone(),
                None => {
                    return json!({
                        "tool": "iso.fetch",
                        "status": "error",
                        "reason": "catalog entry has no download_url"
                    });
                }
            }
        }
    } else {
        return json!({
            "tool": "iso.fetch",
            "status": "error",
            "reason": "provide either --id <catalog-id> or --url <https://...>"
        });
    };

    let filename = sanitize_iso_filename(&resolved_url, params.id.as_deref());
    let dest_path = dest_dir.join(&filename);

    if params.url_only {
        return json!({
            "tool": "iso.fetch",
            "status": "ok",
            "mode": "url_only",
            "id": params.id,
            "filename": filename,
            "dest_path": dest_path.display().to_string(),
            "url": resolved_url,
            "cached_url_hit": cached_url_hit,
        });
    }

    if let Some(blocked) = gate("iso.fetch", params.confirm) {
        return blocked;
    }

    if let Err(e) = tokio::fs::create_dir_all(&dest_dir).await {
        return json!({
            "tool": "iso.fetch",
            "status": "error",
            "reason": format!("create dest_dir failed: {e}")
        });
    }

    let part_path = dest_dir.join(format!("{filename}.part"));
    let client = match reqwest::Client::builder()
        .user_agent("vmcraft-iso-fetcher/0.1")
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return json!({"tool": "iso.fetch", "status": "error", "reason": format!("http client: {e}")});
        }
    };

    let download_fut = async {
        let mut resp = client
            .get(&resolved_url)
            .send()
            .await
            .map_err(|e| format!("request failed: {e}"))?;
        if !resp.status().is_success() {
            return Err(format!("HTTP {}", resp.status()));
        }
        let mut out = tokio::fs::File::create(&part_path)
            .await
            .map_err(|e| format!("create part file: {e}"))?;
        let mut hasher = Sha256::new();
        let mut total_bytes: u64 = 0;

        while let Some(chunk) = resp
            .chunk()
            .await
            .map_err(|e| format!("stream chunk failed: {e}"))?
        {
            hasher.update(&chunk);
            out.write_all(&chunk)
                .await
                .map_err(|e| format!("write failed: {e}"))?;
            total_bytes += chunk.len() as u64;
        }
        out.flush()
            .await
            .map_err(|e| format!("flush failed: {e}"))?;
        drop(out);

        if dest_path.exists() {
            let _ = tokio::fs::remove_file(&dest_path).await;
        }
        tokio::fs::rename(&part_path, &dest_path)
            .await
            .map_err(|e| format!("rename part -> iso failed: {e}"))?;

        let sha256 = format!("{:x}", hasher.finalize());
        let sidecar = PathBuf::from(format!("{}.sha256", dest_path.display()));
        let _ = tokio::fs::write(&sidecar, format!("{sha256}  {filename}\n")).await;

        Ok::<(u64, String), String>((total_bytes, sha256))
    };

    match with_timeout_secs("iso.fetch", 1800, download_fut).await {
        Ok(Ok((size_bytes, sha256))) => json!({
            "tool": "iso.fetch",
            "status": "ok",
            "id": params.id,
            "filename": filename,
            "path": dest_path.display().to_string(),
            "size_bytes": size_bytes,
            "size_mb": (size_bytes as f64 / (1024.0 * 1024.0) * 10.0).round() / 10.0,
            "sha256": sha256,
            "url": resolved_url,
            "cached_url_hit": cached_url_hit,
        }),
        Ok(Err(e)) => {
            let _ = tokio::fs::remove_file(&part_path).await;
            json!({"tool": "iso.fetch", "status": "error", "reason": e, "url": resolved_url})
        }
        Err(e) => {
            let _ = tokio::fs::remove_file(&part_path).await;
            json!({"tool": "iso.fetch", "status": "error", "reason": e.to_string(), "url": resolved_url})
        }
    }
}

pub async fn iso_attach(vm: &str, iso_path: &str, confirm: bool) -> Value {
    if let Some(blocked) = gate("iso.attach", confirm) {
        return blocked;
    }
    if !valid_name(vm, 128) {
        return json!({"tool": "iso.attach", "status": "error", "reason": "invalid vm name"});
    }
    let trimmed_iso = iso_path.trim();
    if trimmed_iso.is_empty() || !Path::new(trimmed_iso).is_file() {
        return json!({
            "tool": "iso.attach",
            "status": "error",
            "reason": format!("ISO file not found: '{trimmed_iso}'")
        });
    }

    let mut last_err = String::new();
    for prog in vboxmanage_candidates() {
        // Detect controller name from showvminfo (prefer IDE, fallback SATA)
        let controller =
            match run_capture(&prog, &["showvminfo", vm, "--machinereadable"], 15).await {
                Ok(info) => detect_optical_controller(&info).unwrap_or_else(|| "IDE".to_string()),
                Err(e) => {
                    last_err = e;
                    continue;
                }
            };

        let args = [
            "storageattach",
            vm,
            "--storagectl",
            &controller,
            "--port",
            "1",
            "--device",
            "0",
            "--type",
            "dvddrive",
            "--medium",
            trimmed_iso,
        ];
        match run_capture(&prog, &args, 30).await {
            Ok(_) => {
                return json!({
                    "tool": "iso.attach",
                    "status": "ok",
                    "vm": vm,
                    "controller": controller,
                    "iso_path": trimmed_iso,
                });
            }
            Err(e) => {
                // Retry on port 0 if port 1 isn't present on that controller
                let args_p0 = [
                    "storageattach",
                    vm,
                    "--storagectl",
                    &controller,
                    "--port",
                    "0",
                    "--device",
                    "0",
                    "--type",
                    "dvddrive",
                    "--medium",
                    trimmed_iso,
                ];
                if run_capture(&prog, &args_p0, 30).await.is_ok() {
                    return json!({
                        "tool": "iso.attach",
                        "status": "ok",
                        "vm": vm,
                        "controller": controller,
                        "port": 0,
                        "iso_path": trimmed_iso,
                    });
                }
                last_err = e;
            }
        }
    }
    down("iso.attach", &last_err)
}

fn detect_optical_controller(machinereadable: &str) -> Option<String> {
    let mut fallback: Option<String> = None;
    for line in machinereadable.lines() {
        if let Some(rest) = line.strip_prefix("storagecontrollername") {
            if let Some((_, val)) = rest.split_once('=') {
                let name = val.trim().trim_matches('"').to_string();
                if name.to_ascii_uppercase().contains("IDE") {
                    return Some(name);
                }
                if fallback.is_none() && !name.is_empty() {
                    fallback = Some(name);
                }
            }
        }
    }
    fallback
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_filters_by_category() {
        let all = iso_catalog(None);
        let linux = iso_catalog(Some("linux"));
        let uefi = iso_catalog(Some("uefi"));
        let windows = iso_catalog(Some("windows"));
        assert!(all["count"].as_u64().unwrap_or(0) >= 10);
        assert!(linux["count"].as_u64().unwrap_or(0) >= 6);
        assert!(uefi["count"].as_u64().unwrap_or(0) >= 2);
        assert!(windows["count"].as_u64().unwrap_or(0) >= 2);
    }

    #[test]
    fn sanitize_filename_strips_query_and_traversal() {
        let url = "https://software.download.prss.microsoft.com/dbazure/Windows11_Client_x64_en-gb_26300_9457.iso?t=abc&P1=123";
        assert_eq!(
            sanitize_iso_filename(url, Some("windows-11-latest")),
            "Windows11_Client_x64_en-gb_26300_9457.iso"
        );
        let weird = "https://example.com/../../evil?x=1";
        assert_eq!(sanitize_iso_filename(weird, Some("alpine")), "alpine.iso");
    }

    #[test]
    fn rejects_non_https_urls() {
        assert!(validate_https_url("http://example.com/a.iso").is_err());
        assert!(validate_https_url("file:///C:/a.iso").is_err());
        assert!(validate_https_url("https://example.com/a.iso").is_ok());
    }

    #[test]
    fn fido_cache_roundtrip_and_expiry() {
        let tmp = std::env::temp_dir().join(format!("vmcraft-iso-test-{}", uuid::Uuid::new_v4()));
        let _ = std::fs::create_dir_all(&tmp);
        let now = Utc::now();
        save_fido_cache_hit(&tmp, "win11|en|x64", "https://cdn.example/win11.iso", now);
        assert_eq!(
            load_fido_cache_hit(&tmp, "win11|en|x64", now + ChronoDuration::hours(2)),
            Some("https://cdn.example/win11.iso".to_string())
        );
        assert_eq!(
            load_fido_cache_hit(&tmp, "win11|en|x64", now + ChronoDuration::hours(25)),
            None
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[tokio::test]
    async fn iso_verify_and_list_work_on_temp_iso() {
        let tmp = std::env::temp_dir().join(format!("vmcraft-iso-v-{}", uuid::Uuid::new_v4()));
        let _ = std::fs::create_dir_all(&tmp);
        let iso = tmp.join("tiny-test.iso");
        let _ = std::fs::write(&iso, b"vmcraft-uefi-test-bytes");
        let listed = iso_list(Some(&tmp.display().to_string()));
        assert_eq!(listed["count"].as_u64(), Some(1));

        let verified = iso_verify(&iso.display().to_string(), None).await;
        assert_eq!(verified["status"].as_str(), Some("ok"));
        let digest = verified["sha256"].as_str().unwrap_or("").to_string();
        assert_eq!(digest.len(), 64);

        let matched = iso_verify(&iso.display().to_string(), Some(&digest)).await;
        assert_eq!(matched["verified"].as_bool(), Some(true));

        let mismatch = iso_verify(&iso.display().to_string(), Some("0000")).await;
        assert_eq!(mismatch["verified"].as_bool(), Some(false));
        assert_eq!(mismatch["status"].as_str(), Some("error"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn detects_optical_controller_prefers_ide() {
        let sample = "storagecontrollername0=\"SATA\"\nstoragecontrollername1=\"IDE\"\n";
        assert_eq!(detect_optical_controller(sample).as_deref(), Some("IDE"));
        let sata_only = "storagecontrollername0=\"SATA\"\n";
        assert_eq!(
            detect_optical_controller(sata_only).as_deref(),
            Some("SATA")
        );
    }
}
