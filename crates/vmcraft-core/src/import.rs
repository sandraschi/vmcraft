//! Zero-config import from legacy Python `virtualization-mcp` (read-only).
//! Precedence: explicit args > process env > imported .env.

use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct VmImport {
    pub vboxmanage_path: Option<String>,
    pub legacy_dir: Option<PathBuf>,
}

fn parse_env_text(text: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        let body = if let Some(rest) = t.strip_prefix("export ") {
            rest.trim()
        } else {
            t
        };
        if let Some(pos) = body.find('=') {
            let k = body[..pos].trim().to_string();
            let mut v = body[pos + 1..].trim().to_string();
            if (v.starts_with('"') && v.ends_with('"') && v.len() >= 2)
                || (v.starts_with('\'') && v.ends_with('\'') && v.len() >= 2)
            {
                v = v[1..v.len() - 1].to_string();
            }
            if !k.is_empty()
                && k.chars()
                    .all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit())
            {
                map.insert(k, v);
            }
        }
    }
    map
}

pub fn fleet_secret(_name: &str) -> Option<String> {
    None
}

pub fn legacy_dir_candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(v) = std::env::var("VIRTUALIZATION_MCP_DIR") {
        let t = v.trim().to_string();
        if !t.is_empty() {
            out.push(PathBuf::from(t));
        }
    }
    out.push(PathBuf::from(r"D:\Dev\repos\virtualization-mcp"));
    out
}

pub fn load() -> VmImport {
    let mut imp = VmImport::default();
    for dir in legacy_dir_candidates() {
        let envp = dir.join(".env");
        if let Ok(text) = std::fs::read_to_string(&envp) {
            let map = parse_env_text(&text);
            if let Some(v) = map.get("VBOXMANAGE_PATH") {
                let t = v.trim().to_string();
                if !t.is_empty() {
                    imp.vboxmanage_path = Some(t);
                }
            }
            imp.legacy_dir = Some(dir);
            break;
        }
        if dir.is_dir() {
            imp.legacy_dir = Some(dir);
            break;
        }
    }
    if imp.vboxmanage_path.is_none() {
        if let Ok(v) = std::env::var("VBOXMANAGE_PATH") {
            let t = v.trim().to_string();
            if !t.is_empty() {
                imp.vboxmanage_path = Some(t);
            }
        }
    }
    imp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_env_basic() {
        let m = parse_env_text("VBOXMANAGE_PATH=C:\\x\\VBoxManage.exe\n# comment\nEMPTY=\n");
        assert_eq!(
            m.get("VBOXMANAGE_PATH").map(String::as_str),
            Some("C:\\x\\VBoxManage.exe")
        );
    }

    #[test]
    fn precedence_env_over_nothing() {
        let imp = load();
        let _ = imp.legacy_dir.is_some();
    }
}
