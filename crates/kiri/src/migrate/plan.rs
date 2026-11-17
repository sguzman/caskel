//! Conservative, deterministic, non-mutating Docker migration preview.
//!
//! This module consumes only the redacted, read-only inventory. It cannot
//! connect to a daemon, create containers, copy secrets or apply its steps.
//! "Potential" conflicts are source-side observations, NOT Caskel readiness.

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::net::IpAddr;

use serde_json::{json, Value};

use super::{array, string, strings};

#[derive(Debug, Clone, PartialEq, Eq)]
struct PortLease {
    container: String,
    address: String,
    port: u16,
    protocol: String,
    state: String,
}

fn wildcard(address: &str) -> bool {
    address.is_empty() || matches!(address, "0.0.0.0" | "::" | "[::]")
}

fn host_port(value: &Value) -> Option<u16> {
    let parsed = value.as_u64()
        .or_else(|| value.as_str().and_then(|v| v.parse::<u64>().ok()))?;
    u16::try_from(parsed).ok().filter(|port| *port != 0)
}

/// Enumerate all configured host bindings, including stopped containers.
/// NetworkSettings ports alone can omit these once the container stops.
fn port_leases(containers: &[Value]) -> Vec<PortLease> {
    let mut leases = Vec::new();
    for container in containers {
        let name = string(container, "name");
        let state = string(container, "state");
        let mut seen = BTreeSet::new();
        for binding in container["configured_port_bindings"].as_array().into_iter().flatten() {
            let protocol = binding["container"].as_str().and_then(|s| s.rsplit_once('/'))
                .map_or("tcp", |(_, protocol)| protocol);
            for h in binding["hosts"].as_array().into_iter().flatten() {
                if let Some(port) = host_port(&h["host_port"]) {
                    let address = h["host_ip"].as_str().unwrap_or_default().to_owned();
                    let key = (protocol.to_owned(), address.clone(), port);
                    if seen.insert(key) {
                        leases.push(PortLease {
                            container: name.clone(), state: state.clone(),
                            address, port, protocol: protocol.to_owned(),
                        });
                    }
                }
            }
        }
        // Some Docker API versions only expose actual published ports.
        for binding in container["published_ports"].as_array().into_iter().flatten() {
            if let Some(port) = host_port(&binding["host_port"]) {
                let protocol = binding["protocol"].as_str().unwrap_or("tcp").to_owned();
                let address = binding["host_ip"].as_str().unwrap_or_default().to_owned();
                let key = (protocol.clone(), address.clone(), port);
                if seen.insert(key) {
                    leases.push(PortLease {
                        container: name.clone(), state: state.clone(),
                        address, port, protocol,
                    });
                }
            }
        }
    }
    leases.sort_by(|a, b| (&a.protocol, a.port, &a.container, &a.address)
        .cmp(&(&b.protocol, b.port, &b.container, &b.address)));
    leases
}

fn source_port_reuse(containers: &[Value]) -> Vec<Value> {
    let leases = port_leases(containers);
    let mut by_port: BTreeMap<(String, u16), Vec<&PortLease>> = BTreeMap::new();
    for lease in &leases {
        by_port.entry((lease.protocol.clone(), lease.port)).or_default().push(lease);
    }
    let mut conflicts = Vec::new();
    for ((protocol, port), group) in by_port {
        let mut overlapping = BTreeSet::new();
        for (i, a) in group.iter().enumerate() {
            for b in group.iter().skip(i + 1) {
                if a.container != b.container
                    && (a.address == b.address || wildcard(&a.address) || wildcard(&b.address))
                {
                    overlapping.insert(a.container.clone());
                    overlapping.insert(b.container.clone());
                }
            }
        }
        if !overlapping.is_empty() {
            conflicts.push(json!({
                "protocol": protocol,
                "host_port": port,
                "containers": overlapping,
                "reason": "source containers configure overlapping host ports; stopped bindings may collide at cutover"
            }));
        }
    }
    conflicts
}

/// Potential overlap in configured source and target published host ports.
///// Even if both engines currently run, this is not a live socket-occupancy
/// probe and must not be interpreted as permission to take over a port.
fn source_target_port_overlap(source: &[Value], target: &[Value]) -> Vec<Value> {
    let source_leases = port_leases(source);
    let target_leases = port_leases(target);
    let mut conflicts = BTreeSet::new();
    for a in &source_leases {
        for b in &target_leases {
            if a.protocol == b.protocol
                && a.port == b.port
                && (a.address == b.address || wildcard(&a.address) || wildcard(&b.address))
            {
                conflicts.insert((a.protocol.clone(), a.port, a.container.clone(),
                    b.container.clone(), a.address.clone(), b.address.clone()));
            }
        }
    }
    conflicts.into_iter().map(|(protocol, port, source, target, source_ip, target_ip)| json!({
        "protocol": protocol,
        "host_port": port,
        "source_container": source,
        "target_container": target,
        "source_host_ip": source_ip,
        "target_host_ip": target_ip,
        "reason": "both engines have configured bindings that may overlap; source release and live host occupancy need verification"
    })).collect()
}

/// Compare two CIDR ranges without assuming that network names are unique
/// across engines. An invalid or unavailable subnet is never interpreted as
/// proof of compatibility; all workloads remain blocked.
fn cidr_overlap(a: &str, b: &str) -> bool {
    fn parse(s: &str) -> Option<(IpAddr, u8)> {
        let (addr, prefix) = s.split_once('/')?;
        let ip: IpAddr = addr.parse().ok()?;
        let bits: u8 = prefix.parse().ok()?;
        if bits > if ip.is_ipv4() { 32 } else { 128 } { return None; }
        Some((ip, bits))
    }
    match (parse(a), parse(b)) {
        (Some((IpAddr::V4(a), x)), Some((IpAddr::V4(b), y))) => {
            let common = x.min(y);
            common == 0 || (u32::from(a) >> (32 - common)) == (u32::from(b) >> (32 - common))
        }
        (Some((IpAddr::V6(a), x)), Some((IpAddr::V6(b), y))) => {
            let common = x.min(y);
            common == 0 || (u128::from(a) >> (128 - common)) == (u128::from(b) >> (128 - common))
        }
        _ => false,
    }
}

/// Source/target network range overlaps are potential routing conflicts,
/// not evidence of the current host routing table or an approved cutover.
fn source_target_subnet_overlap(source: &[Value], target: &[Value]) -> Vec<Value> {
    let mut matches = BTreeSet::new();
    for a in source {
        for source_cidr in strings(&a["subnets"]) {
            for b in target {
                for target_cidr in strings(&b["subnets"]) {
                    if cidr_overlap(&source_cidr, &target_cidr) {
                        matches.insert((
                            string(a, "name"), source_cidr.clone(),
                            string(b, "name"), target_cidr,
                        ));
                    }
                }
            }
        }
    }
    matches.into_iter().map(|(source_network, source_subnet, target_network, target_subnet)| json!({
        "source_network": source_network,
        "source_subnet": source_subnet,
        "target_network": target_network,
        "target_subnet": target_subnet,
        "reason": "configured Docker and Caskel IPAM ranges overlap; actual routes, gateways and migration isolation require review"
    })).collect()
}

fn findings(container: &Value) -> Vec<&'static str> {
    let mut notes = BTreeSet::new();
    notes.insert("image_must_be_transferred_or_pulled_separately");
    notes.insert("recreate_from_original_compose_or_verified_inspect_spec");
    notes.insert("health_and_rollback_verification_required");

    if container["compose_project"].is_string() {
        notes.insert("original_compose_files_and_project_dependencies_required");
    }
    if container["mounts"].as_array().is_some_and(|m| !m.is_empty()) {
        notes.insert("verify_mount_ownership_permissions_and_restore");
    }
    for mount in container["mounts"].as_array().into_iter().flatten() {
        match mount["type"].as_str().unwrap_or("") {
            "bind" => { notes.insert("bind_mount_path_and_source_stop_review"); }
            "volume" => { notes.insert("application_consistent_volume_backup_required"); }
            "tmpfs" => { notes.insert("recreate_tmpfs_without_persisting_ephemeral_data"); }
            _ => { notes.insert("unknown_mount_type_requires_manual_review"); }
        }
    }
    if container["published_ports"].as_array().is_some_and(|v| !v.is_empty())
        || container["configured_port_bindings"].as_array().is_some_and(|v| !v.is_empty())
    {
        notes.insert("published_ports_need_source_release_and_target_collision_check");
    }
    let mode = container["network_mode"].as_str().unwrap_or("");
    if mode != "" && mode != "default" && mode != "bridge" && mode != "none" {
        notes.insert("non_default_network_mode_requires_compatibility_review");
    }
    if container["networks"].as_array().is_some_and(|v| v.iter().any(|n|
        n.as_str().is_some_and(|n| n != "bridge" && n != "none")))
    {
        notes.insert("custom_network_dns_and_ipam_must_be_recreated");
    }
    if container["security"]["privileged"] == true
        || container["security"]["extra_capabilities"] == true
        || container["security"]["devices"] == true
    {
        notes.insert("privileged_capabilities_or_devices_require_security_review");
    }
    if container["healthcheck_configured"] == true {
        notes.insert("replicate_and_validate_original_healthcheck");
    }
    if !matches!(container["restart_policy"].as_str().unwrap_or(""), "no" | "") {
        notes.insert("reapply_restart_policy_after_health_verification");
    }
    notes.into_iter().collect()
}

/// A migration **preview**, never an executable plan. Target Caskel state,
/// live port occupancy, image compatibility and safe database consistency
/// have not been checked; therefore every workload remains blocked.
pub(super) fn preview(inventory: &Value) -> io::Result<Value> {
    preview_optional_target(inventory, None)
}

/// Optional opt-in target inspection remains non-mutating and NEVER marks
/// any workload ready, even if the target is empty or no overlaps are found.
pub(super) fn preview_with_target(inventory: &Value, target: &Value) -> io::Result<Value> {
    preview_optional_target(inventory, Some(target))
}

fn preview_optional_target(inventory: &Value, target: Option<&Value>) -> io::Result<Value> {
    let containers = array(&inventory["containers"], "inventory containers")?;
    let target_containers = match target {
        Some(v) => array(&v["containers"], "target containers")?.as_slice(),
        None => &[],
    };
    let source_networks = array(&inventory["networks"], "source networks")?;
    let target_networks = match target {
        Some(v) => array(&v["networks"], "target networks")?.as_slice(),
        None => &[],
    };
    let mut workloads = Vec::new();
    for c in containers {
        workloads.push(json!({
            "name": string(c, "name"),
            "source_id": string(c, "id"),
            "source_state": string(c, "state"),
            "image": string(c, "image"),
            "compose_project": c["compose_project"],
            "network_mode": string(c, "network_mode"),
            "source_mount_count": c["mounts"].as_array().map_or(0, Vec::len),
            "source_port_binding_count": c["configured_port_bindings"].as_array().map_or(0, Vec::len),
            "source_networks": strings(&c["networks"]),
            "status": "blocked_pending_manual_validation",
            "findings": findings(c),
        }));
    }
    workloads.sort_by(|a, b|
        a["name"].as_str().cmp(&b["name"].as_str()));

    Ok(json!({
        "mode": "read_only_dry_run",
        "source_socket": string(inventory, "source_socket"),
        "docker_version": string(inventory, "docker_version"),
        "target_caskel_inspected": target.is_some(),
        "target_socket": target.map(|v| string(v, "target_socket")),
        "target_container_count": target.map(|_| target_containers.len()),
        "potential_source_target_port_overlap": source_target_port_overlap(containers, target_containers),
        "potential_source_target_subnet_overlap": source_target_subnet_overlap(source_networks, target_networks),
        "target_network_count": target.map(|_| target_networks.len()),
        "ready_to_apply": false,
        "workload_count": workloads.len(),
        "compose_projects": strings(&inventory["compose_projects"]),
        "potential_source_port_reuse": source_port_reuse(containers),
        "workloads": workloads,
        "unverified_requirements": [
            "target Caskel image/platform support, capabilities, DNS, gateways and network/IPAM compatibility not fully checked; missing or malformed subnet data cannot establish safety",
            "live host port occupancy (including non-container processes) not checked; source ports must be released deliberately",
            "target storage ownership, permissions and available space not checked",
            "database and other active write-heavy services require application-consistent backup and tested restore",
            "Compose secrets, dependency order and external resources need original Compose/configuration review",
            "a controlled source stop, target health check and Docker rollback are mandatory before cutover"
        ],
        "proposed_manual_sequence": [
            "Review this private report and the original Docker Compose/configuration",
            "Confirm target compatibility and test on disposable services",
            "Prepare and verify application-consistent backups and restore",
            "Transfer or pull verified images into isolated Caskel state",
            "Recreate networks and volumes without sharing live writable database directories",
            "Stop one selected Docker service only during a separately authorized cutover",
            "Recreate and health-check that service in Caskel; verify DNS/ports and data",
            "If unhealthy, stop Caskel copy and restore the original Docker Compose service"
        ],
        "message": "No Docker or Caskel state was modified. This report cannot authorize a cutover."
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Value {
        json!({
            "source_socket": "/var/run/docker.sock", "docker_version": "29.0",
            "compose_projects": ["demo"], "networks": [],
            "containers": [
                {
                    "id": "ab01", "name": "db", "state": "running", "image": "postgres:16",
                    "compose_project": "demo", "restart_policy": "unless-stopped",
                    "mounts": [{"type": "volume", "source": "/data/docker/db", "destination": "/db", "read_write": true}],
                    "published_ports": [],
                    "configured_port_bindings": [{"container": "5432/tcp", "hosts": [{"host_ip": "127.0.0.1", "host_port": "15432"}]}],
                    "networks": ["demo_net"], "network_mode": "demo_net",
                    "security": {"privileged": false, "extra_capabilities": false, "devices": false},
                    "healthcheck_configured": true
                },
                {
                    "id": "ab02", "name": "backup", "state": "exited", "image": "backup:latest",
                    "compose_project": null, "restart_policy": "no",
                    "mounts": [{"type": "bind", "source": "/srv/backups", "destination": "/backups", "read_write": true}],
                    "published_ports": [],
                    "configured_port_bindings": [{"container": "80/tcp", "hosts": [{"host_ip": "0.0.0.0", "host_port": "15432"}]}],
                    "networks": ["bridge"], "network_mode": "bridge",
                    "security": {"privileged": true, "extra_capabilities": false, "devices": false}
                }
            ]
        })
    }

    #[test]
    fn preview_is_deterministic_and_never_ready_for_cutover() {
        let source = fixture();
        let one = preview(&source).unwrap();
        assert_eq!(one, preview(&source).unwrap());
        assert_eq!(one["mode"], "read_only_dry_run");
        assert_eq!(one["target_caskel_inspected"], false);
        assert_eq!(one["ready_to_apply"], false);
        assert_eq!(one["workload_count"], 2);
        for workload in one["workloads"].as_array().unwrap() {
            assert_eq!(workload["status"], "blocked_pending_manual_validation");
        }
        assert!(one["proposed_manual_sequence"].as_array().unwrap().len() >= 7);
    }

    #[test]
    fn reports_stopped_source_port_reuse_and_storage_security_risks() {
        let plan = preview(&fixture()).unwrap();
        let collisions = plan["potential_source_port_reuse"].as_array().unwrap();
        assert_eq!(collisions.len(), 1);
        assert_eq!(collisions[0]["host_port"], 15432);
        assert_eq!(collisions[0]["containers"], json!(["backup", "db"]));
        let db = &plan["workloads"][1];
        let notes = db["findings"].as_array().unwrap();
        assert!(notes.iter().any(|v| v == "application_consistent_volume_backup_required"));
        assert!(notes.iter().any(|v| v == "custom_network_dns_and_ipam_must_be_recreated"));
        let backup = &plan["workloads"][0];
        let checks = backup["findings"].as_array().unwrap();
        assert!(checks.iter().any(|v| v == "bind_mount_path_and_source_stop_review"));
        assert!(checks.iter().any(|v| v == "privileged_capabilities_or_devices_require_security_review"));
    }

    #[test]
    fn does_not_copy_raw_inspect_secrets() {
        let mut source = fixture();
        source["containers"][0]["secrets"] = json!({"env": "SENSITIVE-SECRET"});
        source["containers"][0]["labels"] = json!({"token": "SENSITIVE-SECRET"});
        let report = preview(&source).unwrap().to_string();
        assert!(!report.contains("SENSITIVE-SECRET"));
    }

    #[test]
    fn optional_target_flags_cross_engine_overlap_but_never_approves_cutover() {
        let mut source = fixture();
        source["networks"] = json!([]);
        let target = json!({
            "target_socket": "/run/caskel/docker.sock",
            "networks": [],
            "containers": [{
                "name": "existing", "state": "exited",
                "configured_port_bindings": [{
                    "container": "9000/tcp",
                    "hosts": [{"host_ip": "0.0.0.0", "host_port": "15432"}]
                }],
                "published_ports": []
            }]
        });
        let plan = preview_with_target(&source, &target).unwrap();
        assert_eq!(plan, preview_with_target(&source, &target).unwrap());
        assert_eq!(plan["target_caskel_inspected"], true);
        assert_eq!(plan["target_container_count"], 1);
        assert_eq!(plan["ready_to_apply"], false);
        assert_eq!(plan["potential_source_target_port_overlap"].as_array().unwrap().len(), 2);
        assert_eq!(plan["workloads"][0]["status"], "blocked_pending_manual_validation");
    }

    #[test]
    fn cidr_overlap_covers_ipv4_ipv6_and_rejects_bad_subnets() {
        assert!(cidr_overlap("10.10.0.0/16", "10.10.1.0/24"));
        assert!(!cidr_overlap("10.10.0.0/16", "10.11.0.0/16"));
        assert!(cidr_overlap("0.0.0.0/0", "192.0.2.0/24"));
        assert!(cidr_overlap("fd00:abcd::/48", "fd00:abcd:12::/64"));
        assert!(!cidr_overlap("fd00:abcd::/48", "fd00:abce::/48"));
        assert!(!cidr_overlap("10.10.0.0/16", "fd00::/64"));
        assert!(!cidr_overlap("10.10.0.0/33", "10.10.0.0/24"));
        assert!(!cidr_overlap("fd00::/129", "fd00::/64"));
        assert!(!cidr_overlap("not-a-cidr", "10.10.0.0/24"));
    }

    #[test]
    fn target_subnets_warn_deterministically_without_approving_migration() {
        let mut source = fixture();
        source["networks"] = json!([
            {"name": "source-net", "subnets": ["10.22.0.0/16", "fd00:beef::/48"]},
            {"name": "isolated", "subnets": ["10.100.0.0/16"]}
        ]);
        let target = json!({
            "target_socket": "/run/caskel/docker.sock",
            "containers": [],
            "networks": [
                {"name": "other", "subnets": ["fd00:beef:1::/64", "10.22.10.0/24"]},
                {"name": "invalid", "subnets": ["not-a-cidr"]}
            ]
        });
        let report = preview_with_target(&source, &target).unwrap();
        assert_eq!(report, preview_with_target(&source, &target).unwrap());
        assert_eq!(report["target_network_count"], 2);
        assert_eq!(report["ready_to_apply"], false);
        assert_eq!(report["potential_source_target_subnet_overlap"].as_array().unwrap().len(), 2);
        assert!(report["workloads"].as_array().unwrap().iter()
            .all(|c| c["status"] == "blocked_pending_manual_validation"));
        assert!(preview(&source).unwrap()["potential_source_target_subnet_overlap"].as_array().unwrap().is_empty());
    }

    #[test]
    fn distinct_bind_addresses_do_not_conflict_without_wildcard() {
        let mut source = fixture();
        source["containers"][1]["configured_port_bindings"][0]["hosts"][0]["host_ip"] = json!("192.0.2.15");
        assert!(preview(&source).unwrap()["potential_source_port_reuse"].as_array().unwrap().is_empty());
    }
}
