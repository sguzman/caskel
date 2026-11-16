//! Read-only Linux Docker Engine discovery, independent of Caskel's socket.
//! Never send write requests, expose Docker Env/label values or change context.

use std::collections::BTreeSet;
use std::io::{self, Read, Write};
use std::os::unix::fs::FileTypeExt;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::{json, Value};

const USAGE: &str = "usage: caskel migrate docker --list [--socket PATH] [--json]";
const DEFAULT_SOCKET: &str = "/var/run/docker.sock";
const MAX_RESPONSE: u64 = 64 * 1024 * 1024;

fn string(v: &Value, key: &str) -> String {
    v[key].as_str().unwrap_or_default().to_owned()
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array().into_iter().flatten().filter_map(|v| v.as_str().map(str::to_owned)).collect()
}

fn array<'a>(v: &'a Value, subject: &str) -> io::Result<&'a Vec<Value>> {
    v.as_array().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
        format!("Docker {subject} did not return an array")))
}

fn source_socket(requested: &Path) -> io::Result<PathBuf> {
    let source = requested.canonicalize().map_err(|e| io::Error::new(e.kind(),
        format!("cannot open Docker source {}: {e}", requested.display())))?;
    if !source.metadata()?.file_type().is_socket() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Docker source must be a Unix socket"));
    }
    let reserved = ["/run/caskel/docker.sock", "/run/kiri/docker.sock"];
    if reserved.iter().filter_map(|p| Path::new(p).canonicalize().ok()).any(|p| p == source)
        || crate::client::socket().ok().and_then(|p| p.canonicalize().ok()).is_some_and(|p| p == source)
    {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,
            "source socket points to Caskel or the old Kiri engine, not Docker"));
    }
    Ok(source)
}

/// Decode content-length/connection-close or chunked HTTP, as Docker can use both.
fn http_response(raw: &[u8]) -> io::Result<(u16, Vec<u8>)> {
    let i = raw.windows(4).position(|w| w == b"\r\n\r\n")
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing HTTP headers"))?;
    let header = std::str::from_utf8(&raw[..i])
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let status = header.lines().next().and_then(|s| s.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing HTTP status"))?;
    let body = &raw[i + 4..];
    let chunked = header.lines().any(|s| s.split_once(':').is_some_and(|(k, v)|
        k.eq_ignore_ascii_case("transfer-encoding") && v.to_ascii_lowercase().contains("chunked")));
    if !chunked {
        return Ok((status, body.to_vec()));
    }
    let mut out = Vec::new();
    let mut at = 0;
    loop {
        let end = body[at..].windows(2).position(|w| w == b"\r\n")
            .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "short HTTP chunk header"))?;
        let size = std::str::from_utf8(&body[at..at + end]).ok()
            .and_then(|s| usize::from_str_radix(s.split(';').next().unwrap_or(""), 16).ok())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid HTTP chunk length"))?;
        at += end + 2;
        if size == 0 {
            return Ok((status, out));
        }
        if size > body.len().saturating_sub(at).saturating_sub(2)
            || body.get(at + size..at + size + 2) != Some(&b"\r\n"[..])
        {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "short HTTP chunk"));
        }
        out.extend_from_slice(&body[at..at + size]);
        at += size + 2;
    }
}

/// The sole transport method. Routes are built locally with validated IDs.
fn get(socket: &Path, route: &str) -> io::Result<Value> {
    if !route.starts_with('/') || !route.is_ascii() || route.bytes().any(|c| c < 0x20) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid Docker API route"));
    }
    let mut conn = UnixStream::connect(socket)?;
    conn.set_read_timeout(Some(Duration::from_secs(30)))?;
    conn.set_write_timeout(Some(Duration::from_secs(30)))?;
    conn.write_all(format!("GET {route} HTTP/1.1\r\nHost: docker\r\nAccept: application/json\r\nConnection: close\r\n\r\n").as_bytes())?;
    let mut raw = Vec::new();
    (&mut conn).take(MAX_RESPONSE + 1).read_to_end(&mut raw)?;
    if raw.len() as u64 > MAX_RESPONSE {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Docker response exceeds 64 MiB"));
    }
    let (status, body) = http_response(&raw)?;
    if status != 200 {
        return Err(io::Error::other(format!("Docker GET {route}: HTTP {status}: {}",
            String::from_utf8_lossy(&body).chars().take(160).collect::<String>())));
    }
    serde_json::from_slice(&body)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Docker GET {route}: {e}")))
}

fn discover(socket: &Path) -> io::Result<Value> {
    let version = get(socket, "/version")?;
    let listing = get(socket, "/containers/json?all=1")?;
    let images = get(socket, "/images/json")?;
    let volumes = get(socket, "/volumes")?;
    let networks = get(socket, "/networks")?;
    let mut containers = Vec::new();
    let mut projects = BTreeSet::new();
    for row in array(&listing, "containers")? {
        let id = string(row, "Id");
        if id.is_empty() || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid Docker container ID"));
        }
        let detail = get(socket, &format!("/containers/{id}/json"))?;
        let labels = &detail["Config"]["Labels"];
        let project = labels["com.docker.compose.project"].as_str().map(str::to_owned);
        if let Some(p) = &project {
            projects.insert(p.clone());
        }
        // Only include label names; a Docker label may contain sensitive data.
        let mut label_keys = labels.as_object().into_iter().flat_map(|o| o.keys().cloned()).collect::<Vec<_>>();
        label_keys.sort_unstable();
        let mounts = detail["Mounts"].as_array().into_iter().flatten().map(|m| json!({
            "type": string(m, "Type"),
            "source": string(m, "Source"),
            "destination": string(m, "Destination"),
            "read_write": m["RW"].as_bool().unwrap_or(false)
        })).collect::<Vec<_>>();
        let ports = row["Ports"].as_array().into_iter().flatten().map(|p| json!({
            "container_port": p["PrivatePort"],
            "host_port": p["PublicPort"],
            "host_ip": p["IP"],
            "protocol": p["Type"]
        })).collect::<Vec<_>>();
        let mut attached_networks = detail["NetworkSettings"]["Networks"].as_object()
            .into_iter().flat_map(|o| o.keys().cloned()).collect::<Vec<_>>();
        attached_networks.sort_unstable();
        let name = strings(&row["Names"]).into_iter().next().unwrap_or_default();
        containers.push(json!({
            "id": id,
            "name": name.trim_start_matches('/'),
            "image": string(row, "Image"),
            "state": string(row, "State"),
            "compose_project": project,
            "restart_policy": string(&detail["HostConfig"]["RestartPolicy"], "Name"),
            "label_keys": label_keys,
            "mounts": mounts,
            "published_ports": ports,
            "networks": attached_networks
        }));
    }
    containers.sort_unstable_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));

    let images = array(&images, "images")?.iter().map(|v| json!({
        "id": string(v, "Id"), "tags": strings(&v["RepoTags"]), "size_bytes": v["Size"]
    })).collect::<Vec<_>>();
    let empty = Vec::new();
    let volume_rows = if volumes["Volumes"].is_null() { &empty } else { array(&volumes["Volumes"], "volumes")? };
    let volumes = volume_rows.iter().map(|v| json!({
        "name": string(v, "Name"), "driver": string(v, "Driver"),
        "mountpoint": string(v, "Mountpoint")
    })).collect::<Vec<_>>();
    let networks = array(&networks, "networks")?.iter().map(|v| json!({
        "name": string(v, "Name"), "id": string(v, "Id"),
        "driver": string(v, "Driver"), "internal": v["Internal"]
    })).collect::<Vec<_>>();
    Ok(json!({
        "source_socket": socket.display().to_string(),
        "docker_version": string(&version, "Version"),
        "containers": containers, "images": images, "volumes": volumes,
        "networks": networks, "compose_projects": projects.into_iter().collect::<Vec<_>>()
    }))
}

pub fn run(args: &[String]) -> io::Result<()> {
    if args == ["--help"] || args == ["-h"] {
        println!("{USAGE}");
        return Ok(());
    }
    if args.first().map(String::as_str) != Some("docker") {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, USAGE));
    }
    let mut socket = PathBuf::from(DEFAULT_SOCKET);
    let mut list = false;
    let mut json_output = false;
    let mut changed_socket = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--list" if !list => list = true,
            "--json" if !json_output => json_output = true,
            "--socket" if !changed_socket => {
                i += 1;
                let s = args.get(i).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, USAGE))?;
                if s.is_empty() || s.starts_with('-') {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, USAGE));
                }
                socket = PathBuf::from(s);
                changed_socket = true;
            }
            _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, USAGE))
        }
        i += 1;
    }
    if !list {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, USAGE));
    }
    let socket = source_socket(&socket)?;
    let inventory = discover(&socket)?;
    if json_output {
        println!("{}", serde_json::to_string_pretty(&inventory).map_err(io::Error::other)?);
    } else {
        println!("Docker Engine inventory (READ ONLY): {}", socket.display());
        println!("Version: {}", inventory["docker_version"].as_str().unwrap_or("?"));
        for key in ["containers", "images", "volumes", "networks", "compose_projects"] {
            println!("{}: {}", key, inventory[key].as_array().map_or(0, Vec::len));
        }
        for c in array(&inventory["containers"], "inventory containers")? {
            let name = c["name"].as_str().unwrap_or("");
            let state = c["state"].as_str().unwrap_or("");
            let safe = |s: &str| s.chars().filter(|c| !c.is_control()).take(50).collect::<String>();
            println!("  {} ({}) [{}]", safe(name), safe(state), safe(&string(c, "image")));
        }
        println!("No Docker state was changed. Dry-run and cutover are not implemented.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_plain_and_chunked_http() {
        let (status, body) = http_response(b"HTTP/1.1 200 OK\r\n\r\n{\"ok\":true}").unwrap();
        assert_eq!(status, 200);
        assert_eq!(body, b"{\"ok\":true}");
        let (status, body) = http_response(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\nb\r\n{\"ok\":true}\r\n0\r\n\r\n").unwrap();
        assert_eq!(status, 200);
        assert_eq!(body, b"{\"ok\":true}");
        assert!(http_response(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\nb\r\nx").is_err());
    }

    #[test]
    fn refuses_implicit_writes_and_unsupported_dry_runs() {
        assert!(run(&["docker".into()]).is_err());
        assert!(run(&["docker".into(), "--dry-run".into()]).is_err());
        assert!(run(&["docker".into(), "--list".into(), "--bad".into()]).is_err());
    }

    #[test]
    fn source_transport_uses_get_only() {
        use std::os::unix::net::UnixListener;
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!("caskel-docker-inventory-{}-{}.sock",
            std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        let listener = UnixListener::bind(&path).unwrap();
        let handle = std::thread::spawn(move || {
            let (mut c, _) = listener.accept().unwrap();
            let mut bytes = [0; 1024];
            let n = c.read(&mut bytes).unwrap();
            assert!(bytes[..n].starts_with(b"GET /version HTTP/1.1\r\n"));
            c.write_all(b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n{\"Version\":\"test\"}").unwrap();
        });
        assert_eq!(get(&path, "/version").unwrap()["Version"], "test");
        handle.join().unwrap();
        std::fs::remove_file(path).unwrap();
    }
}
