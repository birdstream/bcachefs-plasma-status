// SPDX-License-Identifier: GPL-3.0-or-later
use std::{fs, path::Path, process::Command};

struct Filesystem {
    mount: String,
    uuid: String,
    size: String,
    used: String,
    free: String,
    percent: f64,
    online: usize,
    total: usize,
    devices: Vec<Device>,
    reconcile: String,
    reconcile_detail: String,
    scan_pending: String,
    error: String,
}

struct Device { name: String, state: String }

fn quote(s: &str) -> String {
    format!("\"{}\"", s.chars().flat_map(|c| match c {
        '"' => "\\\"".chars().collect::<Vec<_>>(), '\\' => "\\\\".chars().collect(),
        '\n' => "\\n".chars().collect(), '\r' => "\\r".chars().collect(),
        '\t' => "\\t".chars().collect(), c if c < ' ' => " ".chars().collect(),
        c => vec![c],
    }).collect::<String>())
}

fn json(data: &[Filesystem]) -> String {
    let rows = data.iter().map(|f| {
        let devs = f.devices.iter().map(|d| format!("{{\"name\":{},\"state\":{}}}", quote(&d.name), quote(&d.state))).collect::<Vec<_>>().join(",");
        format!("{{\"mount\":{},\"uuid\":{},\"size\":{},\"used\":{},\"free\":{},\"percent\":{},\"online\":{},\"total\":{},\"devices\":[{}],\"reconcile\":{},\"reconcile_detail\":{},\"scan_pending\":{},\"error\":{}}}",
            quote(&f.mount), quote(&f.uuid), quote(&f.size), quote(&f.used), quote(&f.free), f.percent,
            f.online, f.total, devs, quote(&f.reconcile), quote(&f.reconcile_detail), quote(&f.scan_pending), quote(&f.error))
    }).collect::<Vec<_>>().join(",");
    format!("[{rows}]")
}

fn command(args: &[&str], mount: &str) -> Result<String, String> {
    let out = Command::new("bcachefs").args(args).arg(mount).output()
        .map_err(|e| format!("could not start bcachefs: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn field(text: &str, label: &str) -> String {
    text.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        (key.trim() == label).then(|| value.trim().to_string())
    }).unwrap_or_default()
}

fn devices(uuid: &str) -> Vec<Device> {
    let root = Path::new("/sys/fs/bcachefs").join(uuid);
    let Ok(entries) = fs::read_dir(root) else { return vec![] };
    let mut result = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("dev-") { continue }
        let path = entry.path();
        let label = fs::read_to_string(path.join("label")).unwrap_or_default().trim().to_string();
        let state = fs::read_to_string(path.join("state")).unwrap_or_default().trim().to_string();
        let name = if label.is_empty() { name } else { label };
        result.push(Device { name, state });
    }
    result.sort_by(|a,b| a.name.cmp(&b.name));
    result
}

fn scan_pending(uuid: &str) -> String {
    fs::read_to_string(Path::new("/sys/fs/bcachefs").join(uuid).join("reconcile_scan_pending"))
        .unwrap_or_default().trim().to_string()
}

fn mounts() -> Vec<String> {
    let Ok(text) = fs::read_to_string("/proc/self/mounts") else { return vec![] };
    let mut list = Vec::new();
    for line in text.lines() {
        let cols: Vec<_> = line.split_whitespace().collect();
        if cols.len() > 2 && cols[2] == "bcachefs" {
            let mount = cols[1].replace("\\040", " ");
            if !list.contains(&mount) { list.push(mount); }
        }
    }
    list
}

fn inspect(mount: &str) -> Filesystem {
    let usage = command(&["fs", "usage", "-ah"], mount).unwrap_or_default();
    let reconcile = command(&["reconcile", "status"], mount).unwrap_or_default();
    let error = if usage.is_empty() { "Could not read bcachefs fs usage".into() } else { String::new() };
    let uuid = field(&usage, "Filesystem");
    let size = field(&usage, "Size");
    let used = field(&usage, "Used");
    let free = field(&usage, "Free").split_whitespace().next().unwrap_or("").to_string();
    let percent = usage.lines().find(|l| l.trim_start().starts_with("Used:"))
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|v| parse_size(v)).zip(usage.lines().find(|l| l.trim_start().starts_with("Size:"))
            .and_then(|l| l.split_whitespace().nth(1)).and_then(parse_size))
        .map(|(u,s)| if s > 0.0 { u * 100.0 / s } else { 0.0 }).unwrap_or(0.0);
    let devs = devices(&uuid);
    let online = devs.iter().filter(|d| d.state.contains("[rw]") || d.state.contains("[ro]")).count();
    let lines: Vec<_> = reconcile.lines().filter(|l| !l.contains("backtrace") && !l.trim_start().starts_with("[<") ).collect();
    let detail = lines.iter().take(8).map(|s| s.trim()).filter(|s| !s.is_empty()).collect::<Vec<_>>().join("\n");
    let status = if reconcile.contains("waiting:") { "Work waiting" }
        else if reconcile.contains("Scan pending:") { "Active" } else { "Status available" };
    Filesystem { mount: mount.into(), uuid, size, used, free, percent, online,
        total: devs.len(), devices: devs, reconcile: status.into(), reconcile_detail: detail,
        scan_pending: scan_pending(&field(&usage, "Filesystem")), error }
}

fn parse_size(value: &str) -> Option<f64> {
    let n: f64 = value.trim_end_matches(|c: char| !c.is_ascii_digit() && c != '.').parse().ok()?;
    let suffix = value.chars().last()?.to_ascii_uppercase();
    Some(n * match suffix { 'K' => 1024.0, 'M' => 1024.0_f64.powi(2), 'G' => 1024.0_f64.powi(3), 'T' => 1024.0_f64.powi(4), _ => 1.0 })
}

fn main() {
    let watch = std::env::args().any(|a| a == "--watch");
    loop {
        let data: Vec<_> = mounts().iter().map(|m| inspect(m)).collect();
        let json = json(&data);
        if watch {
            let path = std::env::var("BCACHEFS_STATUS_FILE").unwrap_or_else(|_| "/tmp/bcachefs-plasma-status.json".into());
            let tmp = format!("{path}.tmp");
            if fs::write(&tmp, json).is_ok() { let _ = fs::rename(tmp, path); }
            std::thread::sleep(std::time::Duration::from_secs(10));
        } else { println!("{json}"); break; }
    }
}
