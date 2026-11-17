//! Read-only Linux Docker Engine discovery, independent of Caskel's socket.
//! Never send write requests, expose Docker Env/label values or change context.

use std::collections::BTreeSet;
use std::io::{self, Read, Write};
use std::os::unix::fs::FileTypeExt;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::{json, Value};

mod plan;

const USAGE: &str = "usage: caskel migrate docker (--list | --dry-run) [--socket PATH] [--target-socket PATH] [--json]";
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

/// A target is opted in explicitly. It must be a distinct socket, not a
/// variant spelling of the Docker source path.
fn target_socket(requested: &Path, source: &Path) -> io::Result<PathBuf> {
    let target = requested.canonicalize().map_err(|e| io::Error::new(e.kind(),
        format!("cannot open Caskel target {}: {e}", requested.display())))?;
    if !target.metadata()?.file_type().is_socket() || target == source {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,
            "Caskel target must be a distinct Unix socket, not the Docker source"));
    }
    Ok(target)
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

fn ensure_docker_engine(version: &Value) -> io::Result<()> {
    // The older Kiri and current Caskel API advertise KiriVersion even when
    // their Docker-compatible Version field looks like Docker itself.
    let vendor = version["Platform"]["Name"].as_str().unwrap_or_default();
    let ours = version.get("KiriVersion").is_some()
        || version.get("CaskelVersion").is_some()
        || ["kiri", "caskel"].iter().any(|n| vendor.eq_ignore_ascii_case(n))
        || version["Components"].as_array().is_some_and(|components|
            components.iter().any(|c| ["kiri", "caskel"].iter().any(|n|
                c["Name"].as_str().is_some_and(|name| name.eq_ignore_ascii_case(n)))));
    if ours {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,
            "source API identifies as Caskel/Kiri, not Docker Engine"));
    }
    Ok(())
}

/// The requested destination must be Caskel, never a second Docker/Kiri
/// installation. This gate is checked before reading its container list.
fn ensure_caskel_target(version: &Value) -> io::Result<()> {
    if version["Platform"]["Name"].as_str().is_some_and(|v| v.eq_ignore_ascii_case("caskel"))
        && version["CaskelVersion"].as_str().is_some()
    {
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidInput,
            "target API is not an identifiable Caskel engine"))
    }
}

/// A deliberately small target snapshot. Only GET requests and information
/// needed for potential host-port conflicts; never read env or label values.
fn discover_target(socket: &Path) -> io::Result<Value> {
    let version = get(socket, "/version")?;
    ensure_caskel_target(&version)?;
    let listing = get(socket, "/containers/json?all=1")?;
    let mut containers = Vec::new();
    for row in array(&listing, "target containers")? {
        let id = string(row, "Id");
        if id.is_empty() || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                "invalid target container ID"));
        }
        let detail = get(socket, &format!("/containers/{id}/json"))?;
        let mut configured_port_bindings = detail["HostConfig"]["PortBindings"].as_object()
            .into_iter().flat_map(|ports| ports.iter()).map(|(target, hosts)| json!({
                "container": target,
                "hosts": hosts.as_array().into_iter().flatten().map(|host| json!({
                    "host_ip": host["HostIp"], "host_port": host["HostPort"]
                })).collect::<Vec<_>>()
            })).collect::<Vec<_>>();
        configured_port_bindings.sort_unstable_by(|a, b|
            a["container"].as_str().cmp(&b["container"].as_str()));
        let published_ports = row["Ports"].as_array().into_iter().flatten().map(|p| json!({
            "container_port": p["PrivatePort"], "host_port": p["PublicPort"],
            "host_ip": p["IP"], "protocol": p["Type"]
        })).collect::<Vec<_>>();
        let name = strings(&row["Names"]).into_iter().next().unwrap_or_default();
        containers.push(json!({
            "name": name.trim_start_matches('/'),
            "state": string(row, "State"),
            "configured_port_bindings": configured_port_bindings,
            "published_ports": published_ports
        }));
    }
    containers.sort_unstable_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    Ok(json!({
        "target_socket": socket.display().to_string(),
        "target_version": string(&version, "CaskelVersion"),
        "containers": containers
    }))
}

fn discover(socket: &Path) -> io::Result<Value> {
    let version = get(socket, "/version")?;
    ensure_docker_engine(&version)?;
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
        // Also capture configured published-port bindings for stopped
        // containers; docker ps can omit their inactive published ports.
        let mut configured_port_bindings = detail["HostConfig"]["PortBindings"].as_object()
            .into_iter().flat_map(|ports| ports.iter()).map(|(target, hosts)| json!({
                "container": target,
                "hosts": hosts.as_array().into_iter().flatten().map(|host| json!({
                    "host_ip": host["HostIp"], "host_port": host["HostPort"]
                })).collect::<Vec<_>>()
            })).collect::<Vec<_>>();
        configured_port_bindings.sort_unstable_by(|a, b|
            a["container"].as_str().cmp(&b["container"].as_str()));
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
            "network_mode": string(&detail["HostConfig"], "NetworkMode"),
            "healthcheck_configured": detail["Config"]["Healthcheck"].is_object(),
            "security": {
                "privileged": detail["HostConfig"]["Privileged"] == true,
                "extra_capabilities": detail["HostConfig"]["CapAdd"].as_array().is_some_and(|v| !v.is_empty()),
                "devices": detail["HostConfig"]["Devices"].as_array().is_some_and(|v| !v.is_empty())
                    || detail["HostConfig"]["DeviceRequests"].as_array().is_some_and(|v| !v.is_empty()),
            },
            "label_keys": label_keys,
            "mounts": mounts,
            "published_ports": ports,
            "configured_port_bindings": configured_port_bindings,
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
    let mut requested_target: Option<PathBuf> = None;
    let mut list = false;
    let mut dry_run = false;
    let mut json_output = false;
    let mut changed_socket = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--list" if !list && !dry_run => list = true,
            "--dry-run" if !list && !dry_run => dry_run = true,
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
            "--target-socket" if requested_target.is_none() => {
                i += 1;
                let path = args.get(i).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, USAGE))?;
                if path.is_empty() || path.starts_with('-') {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, USAGE));
                }
                requested_target = Some(PathBuf::from(path));
            }
            _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, USAGE))
        }
        i += 1;
    }
    if (!list && !dry_run) || (list && requested_target.is_some()) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, USAGE));
    }
    let socket = source_socket(&socket)?;
    let inventory = discover(&socket)?;
    let result = if dry_run {
        if let Some(requested) = requested_target {
            let target = target_socket(&requested, &socket)?;
            let destination = discover_target(&target)?;
            plan::preview_with_target(&inventory, &destination)?
        } else {
            plan::preview(&inventory)?
        }
    } else {
        inventory
    };
    if json_output {
        println!("{}", serde_json::to_string_pretty(&result).map_err(io::Error::other)?);
    } else if dry_run {
        println!("Caskel migration dry-run (READ ONLY): {}", socket.display());
        println!("{} workloads; target Caskel inspected: {} (no readiness approval).",
            result["workload_count"].as_u64().unwrap_or_default(),
            result["target_caskel_inspected"].as_bool().unwrap_or(false));
        println!("{} potential source-to-target host-port conflicts.",
            result["potential_source_target_port_overlap"].as_array().map_or(0, Vec::len));
        println!("{} possible source-side host-port collisions.",
            result["potential_source_port_reuse"].as_array().map_or(0, Vec::len));
        for c in array(&result["workloads"], "preview workloads")? {
            let safe = |s: &str| s.chars().filter(|c| !c.is_control()).take(48).collect::<String>();
            println!("  {}: blocked pending manual checks ({} findings)",
                safe(c["name"].as_str().unwrap_or("")),
                c["findings"].as_array().map_or(0, Vec::len));
        }
        println!("No workload is approved to cut over. Use --json for risks, proposed checks and rollback sequence.");
    } else {
        println!("Docker Engine inventory (READ ONLY): {}", socket.display());
        println!("Version: {}", result["docker_version"].as_str().unwrap_or("?"));
        for key in ["containers", "images", "volumes", "networks", "compose_projects"] {
            println!("{}: {}", key, result[key].as_array().map_or(0, Vec::len));
        }
        for c in array(&result["containers"], "inventory containers")? {
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
    fn rejects_caskel_and_kiri_compatibility_apis_even_on_other_sockets() {
        let valid = serde_json::json!({"Platform": {"Name": "Docker Engine - Community"}, "Version": "29"});
        assert!(ensure_docker_engine(&valid).is_ok());
        let legacy = serde_json::json!({"Version": "27.5.1", "KiriVersion": "0.2.1"});
        assert!(ensure_docker_engine(&legacy).is_err());
        let renamed = serde_json::json!({"Platform": {"Name": "Caskel"}});
        assert!(ensure_docker_engine(&renamed).is_err());
        let version_only = serde_json::json!({"Version": "27.5.1", "CaskelVersion": "0.2.1"});
        assert!(ensure_docker_engine(&version_only).is_err());
        let component = serde_json::json!({"Components": [{"Name": "Kiri", "Version": "0.2.1"}]});
        assert!(ensure_docker_engine(&component).is_err());
    }

    #[test]
    fn refuses_implicit_writes_and_unsupported_apply() {
        assert!(run(&["docker".into()]).is_err());
        assert!(run(&["docker".into(), "--apply".into()]).is_err());
        // A write request must be rejected even when combined with the
        // otherwise valid read-only modes, before opening any Docker socket.
        assert!(run(&["docker".into(), "--dry-run".into(), "--apply".into()]).is_err());
        assert!(run(&["docker".into(), "--list".into(), "--apply".into()]).is_err());
        assert!(run(&["docker".into(), "--dry-run".into(), "--socket".into(), "".into()]).is_err());
        assert!(run(&["docker".into(), "--list".into(), "--dry-run".into()]).is_err());
        assert!(run(&["docker".into(), "--list".into(), "--bad".into()]).is_err());
        assert!(run(&["docker".into(), "--list".into(), "--target-socket".into(), "/run/caskel/docker.sock".into()]).is_err());
        assert!(run(&["docker".into(), "--dry-run".into(), "--target-socket".into(), "".into()]).is_err());
        assert!(run(&["docker".into(), "--dry-run".into(), "--target-socket".into(), "/tmp/caskel".into(), "--target-socket".into(), "/tmp/other".into()]).is_err());
    }

    #[test]
    fn target_version_rejects_docker_and_old_kiri() {
        assert!(ensure_caskel_target(&json!({"Platform": {"Name": "Caskel"}, "CaskelVersion": "0.0.0"})).is_ok());
        assert!(ensure_caskel_target(&json!({"Platform": {"Name": "Docker Engine"}, "Version": "29"})).is_err());
        assert!(ensure_caskel_target(&json!({"Platform": {"Name": "Kiri"}, "KiriVersion": "0.2"})).is_err());
    }

    #[test]
    fn discovers_populated_docker_metadata_without_secret_values() {
        use std::os::unix::net::UnixListener;
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!("caskel-inventory-fixture-{}-{}.sock",
            std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        let listener = UnixListener::bind(&path).unwrap();
        let handle = std::thread::spawn(move || {
            let replies = [
                ("/version", serde_json::json!({"Version": "29.0.0"})),
                ("/containers/json?all=1", serde_json::json!([{
                    "Id": "abcdef", "Names": ["/database"], "Image": "postgres:16",
                    "State": "running", "Ports": [{"PrivatePort": 5432, "PublicPort": 15432, "IP": "127.0.0.1", "Type": "tcp"}]
                }])),
                ("/images/json", serde_json::json!([{"Id": "sha256:img", "RepoTags": ["postgres:16"], "Size": 1000}])),
                ("/volumes", serde_json::json!({"Volumes": [{"Name": "db-data", "Driver": "local", "Mountpoint": "/var/lib/docker/volumes/db-data"}]})),
                ("/networks", serde_json::json!([{"Id": "netid", "Name": "project_net", "Driver": "bridge", "Internal": false}])),
                ("/containers/abcdef/json", serde_json::json!({
                    "Config": {"Env": ["PASSWORD=DO_NOT_EXPOSE"], "Labels": {
                        "com.docker.compose.project": "demo", "api-token": "DO_NOT_EXPOSE"
                    }},
                    "Mounts": [{"Type": "volume", "Source": "/var/lib/docker/volumes/db-data", "Destination": "/data", "RW": true}],
                    "HostConfig": {"RestartPolicy": {"Name": "unless-stopped"}, "PortBindings": {"5432/tcp": [{"HostIp": "127.0.0.1", "HostPort": "15432"}]}},
                    "NetworkSettings": {"Networks": {"project_net": {}}}
                })),
            ];
            for (expected, value) in replies {
                let (mut conn, _) = listener.accept().unwrap();
                let mut request = [0; 2048];
                let n = conn.read(&mut request).unwrap();
                let request = std::str::from_utf8(&request[..n]).unwrap();
                assert!(request.starts_with(&format!("GET {expected} HTTP/1.1\r\n")), "{request}");
                let body = serde_json::to_vec(&value).unwrap();
                conn.write_all(format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).as_bytes()).unwrap();
                conn.write_all(&body).unwrap();
            }
        });
        let inventory = discover(&path).unwrap();
        assert_eq!(inventory["docker_version"], "29.0.0");
        assert_eq!(inventory["compose_projects"][0], "demo");
        assert_eq!(inventory["containers"][0]["name"], "database");
        assert_eq!(inventory["containers"][0]["restart_policy"], "unless-stopped");
        assert_eq!(inventory["containers"][0]["published_ports"][0]["host_port"], 15432);
        assert_eq!(inventory["containers"][0]["configured_port_bindings"][0]["hosts"][0]["host_port"], "15432");
        assert_eq!(inventory["containers"][0]["networks"][0], "project_net");
        assert_eq!(inventory["containers"][0]["mounts"][0]["destination"], "/data");
        assert_eq!(inventory["images"][0]["tags"][0], "postgres:16");
        assert_eq!(inventory["volumes"][0]["name"], "db-data");
        assert_eq!(inventory["networks"][0]["name"], "project_net");
        assert!(!inventory.to_string().contains("DO_NOT_EXPOSE"),
            "container environment and arbitrary label values are private");
        handle.join().unwrap();
        std::fs::remove_file(path).unwrap();
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
