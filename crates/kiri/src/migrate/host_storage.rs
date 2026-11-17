//! Opt-in host-side metadata for Docker source bind/volume paths.
//!
//! Reads metadata and filesystem capacity only. Never opens file contents,
//! traverses directories, follows a final symlink, mounts, or writes data.
//! The invoking CLI's mount namespace may differ from the Docker host's.

use std::collections::BTreeSet;
use std::ffi::CString;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::Path;

use serde_json::{json, Value};

fn available_bytes(path: &Path) -> Option<u64> {
    let bytes = CString::new(path.as_os_str().as_bytes()).ok()?;
    // SAFETY: statvfs only reads kernel filesystem metadata for a valid
    // NUL-terminated pathname; the initialized output buffer is owned here.
    let mut stats: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(bytes.as_ptr(), &mut stats) } != 0 {
        return None;
    }
    Some((stats.f_bavail as u64).saturating_mul(stats.f_frsize as u64))
}

fn examine(kind: &str, path: &str) -> Value {
    let source = Path::new(path);
    if !source.is_absolute() {
        return json!({"kind": kind, "path": path, "status": "not_absolute"});
    }
    match fs::symlink_metadata(source) {
        Ok(meta) => {
            let ft = meta.file_type();
            let typ = if ft.is_symlink() { "symlink" }
                else if ft.is_dir() { "directory" }
                else if ft.is_file() { "file" }
                else if ft.is_socket() { "socket" }
                else { "other" };
            // Do not follow the final component if it is a symlink; the
            // target and its actual filesystem capacity remain unverified.
            let free = if ft.is_dir() || ft.is_file() { available_bytes(source) } else { None };
            json!({
                "kind": kind, "path": path,
                "status": if ft.is_symlink() { "symlink_not_followed" } else { "observed" },
                "file_type": typ,
                "uid": meta.uid(), "gid": meta.gid(),
                "mode_octal": format!("{:04o}", meta.mode() & 0o7777),
                "filesystem_available_bytes": free
            })
        }
        Err(e) => {
            let status = match e.kind() {
                std::io::ErrorKind::NotFound => "missing",
                std::io::ErrorKind::PermissionDenied => "inaccessible",
                _ => "metadata_error",
            };
            json!({"kind": kind, "path": path, "status": status})
        }
    }
}

/// Report just the source paths in Docker's already-redacted inventory.
/// This does not validate Caskel destination permissions or restoreability.
pub(super) fn snapshot(inventory: &Value) -> Vec<Value> {
    let mut paths = BTreeSet::new();
    for c in inventory["containers"].as_array().into_iter().flatten() {
        for mount in c["mounts"].as_array().into_iter().flatten() {
            let kind = mount["type"].as_str().unwrap_or("");
            if !matches!(kind, "bind" | "volume") { continue; }
            if let Some(path) = mount["source"].as_str().filter(|p| !p.is_empty()) {
                paths.insert((kind.to_owned(), path.to_owned()));
            }
        }
    }
    for v in inventory["volumes"].as_array().into_iter().flatten() {
        if let Some(path) = v["mountpoint"].as_str().filter(|p| !p.is_empty()) {
            paths.insert(("named_volume".to_owned(), path.to_owned()));
        }
    }
    paths.into_iter().map(|(kind, path)| examine(&kind, &path)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;

    #[test]
    fn local_paths_are_metadata_only_and_symlinks_are_not_followed() {
        let root = std::env::temp_dir().join(format!("caskel-storage-metadata-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let file = root.join("marker");
        fs::write(&file, b"DO-NOT-READ-SECRET").unwrap();
        let link = root.join("link");
        symlink(&file, &link).unwrap();
        let missing = root.join("missing");
        let inventory = json!({
            "containers": [{"mounts": [
                {"type": "bind", "source": root.to_string_lossy()},
                {"type": "bind", "source": root.to_string_lossy()},
                {"type": "volume", "source": file.to_string_lossy()},
                {"type": "bind", "source": link.to_string_lossy()},
                {"type": "bind", "source": missing.to_string_lossy()},
                {"type": "tmpfs", "source": "/dev/shm"}
            ]}],
            "volumes": [{"mountpoint": root.to_string_lossy()}]
        });
        let result = snapshot(&inventory);
        assert_eq!(result.len(), 5);
        let as_text = serde_json::to_string(&result).unwrap();
        assert!(!as_text.contains("DO-NOT-READ-SECRET"));
        assert!(!as_text.contains("/dev/shm"));
        assert!(result.iter().any(|v| v["kind"] == "bind" && v["status"] == "missing"));
        assert!(result.iter().any(|v| v["status"] == "symlink_not_followed"
            && v["filesystem_available_bytes"].is_null()));
        assert!(result.iter().any(|v| v["file_type"] == "directory"
            && v["filesystem_available_bytes"].is_u64()));
        assert!(result.iter().any(|v| v["kind"] == "named_volume"));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn bad_path_and_empty_inventory_do_not_imply_readiness() {
        assert_eq!(snapshot(&json!({"containers": [], "volumes": []})), Vec::<Value>::new());
        assert_eq!(examine("bind", "relative/path")["status"], "not_absolute");
    }
}
