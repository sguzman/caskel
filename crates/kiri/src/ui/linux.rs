//! Linux window integrations for Caskel's pinned Nix package, isolated
//! systemd engine, and unsupported legacy Mac/Windows setup mechanisms.

/// Explain an unreachable Caskel engine without acting on the old Kiri
/// installation or modifying a user account.
pub fn engine_down() -> std::io::Error {
    let socket = std::path::Path::new(crate::client::SERVICE_SOCKET);
    std::io::Error::other(match std::os::unix::net::UnixStream::connect(socket) {
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied =>
            "This account cannot access Caskel's socket. Check the isolated caskel.service configuration and authorized docker-group membership.",
        _ if socket.exists() => "Caskel's daemon is not answering. Inspect caskel.service before a deliberate restart.",
        _ => "Caskel's isolated service is not running. Start caskel.service explicitly from a terminal when ready.",
    })
}

pub mod setup {
    use std::io;

    /// The docker CLI comes from the distribution; nothing to set up after it.
    pub fn after_tools() -> io::Result<()> {
        Ok(())
    }

    /// `docker build` through Kiri's BuildKit builder, as on the Mac (needs
    /// the docker-buildx package, which the .deb pulls in).
    pub use crate::builds::ensure_builder;

    pub fn domains() -> io::Result<()> {
        Err(io::Error::other("*.kiri.local isn't on Linux yet; containers answer at <name>.kiri.localhost"))
    }

    pub fn remove_domains() -> io::Result<()> {
        Ok(())
    }

    /// Caskel's service socket is isolated; it never replaces Docker's
    /// /var/run/docker.sock or switches the user's default context.
    pub fn docker_socket_note() -> Option<String> {
        None
    }

    /// Caskel must not select a Docker context from the optional GUI.
    pub fn use_context(_name: &str) -> io::Result<()> {
        Err(io::Error::other(
            "Caskel never changes Docker's default context automatically. Use its dedicated socket explicitly.",
        ))
    }

}

pub mod update {
    use std::io;
    use std::path::PathBuf;
    use std::sync::atomic::AtomicBool;

    const BY_APT: &str = "Caskel updates use its pinned Nix profile; in-app installation is disabled";

    #[derive(serde::Serialize)]
    pub struct Release {
        pub version: String,
    }

    /// Whether `candidate` is a later version than `current`, as on the Mac.
    pub fn newer(candidate: &str, current: &str) -> bool {
        let parts = |v: &str| -> Vec<u64> { v.trim_start_matches('v').split(['.', '-']).map(|p| p.parse().unwrap_or(0)).collect() };
        parts(candidate) > parts(current)
    }

    pub fn latest() -> io::Result<Release> {
        Err(io::Error::other(BY_APT))
    }

    pub fn download(_: &Release, _: &mut dyn FnMut(u64, Option<u64>), _: &AtomicBool) -> io::Result<()> {
        Err(io::Error::other(BY_APT))
    }

    pub fn staged() -> Option<String> {
        None
    }

    pub fn install_staged() -> io::Result<String> {
        Err(io::Error::other(BY_APT))
    }

    pub fn app() -> io::Result<PathBuf> {
        std::env::current_exe()
    }

    /// The Caskel daemon is an opt-in system service. GUI restart is
    /// deliberately disabled until an explicit safe host-control workflow
    /// exists; a restart request must never act on old Kiri or Docker.
    pub fn restart_engine(expect: &str) -> io::Result<String> {
        crate::diag::info!("restarting the engine", version = expect);
        super::onboarding::restart_engine()?;
        let began = std::time::Instant::now();
        let mut last = None;
        while began.elapsed() < std::time::Duration::from_secs(60) {
            if let Some(v) = crate::client::request("GET", "/version", None).ok().filter(|r| r.ok()).and_then(|r| crate::client::kiri_version(&r.json()).map(str::to_owned)) {
                if v == expect {
                    return Ok(v);
                }
                last = Some(v);
            }
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
        Err(io::Error::other(match last {
            Some(v) => format!("Caskel's engine reported {v}, not the requested {expect}; check the pinned Nix profile"),
            None => "Caskel did not respond: inspect caskel.service with journalctl -u caskel.service".to_owned(),
        }))
    }

    /// Never called: the window doesn't start Linux's engine (systemd does).
    pub fn start_engine() -> io::Result<()> {
        Err(super::engine_down())
    }
}

pub mod import {
    use std::io;

    use serde_json::Value;

    /// OrbStack and Docker Desktop are the Mac's; on Linux there's none to list.
    pub fn sources() -> Vec<Value> {
        Vec::new()
    }

    fn none() -> io::Error {
        io::Error::other("importing isn't on Linux")
    }

    pub fn inventory(_: &str) -> io::Result<Value> {
        Err(none())
    }

    pub fn image(_: &str, _: &str) -> io::Result<()> {
        Err(none())
    }

    pub fn volume(_: &str, _: &str) -> io::Result<()> {
        Err(none())
    }

    pub fn container(_: &str, _: &str) -> io::Result<()> {
        Err(none())
    }
}

/// What the window checks when it opens on a Linux host, before anything
/// else, and how it fixes each: the package installs the engine but can't
/// start it for, or give access to, the person who'll use it. The page shows
/// the checklist (index.html, `onboarding`) until every required check passes,
/// so the main window never opens onto an engine it can't reach.
pub mod onboarding {
    use std::io;
    use std::path::Path;
    use std::process::{Command, Stdio};

    const SOCKET: &str = crate::client::SERVICE_SOCKET;
    /// The package's command; pkexec wants a full path.

    /// Each check's state for the page: "ok", "no" (fix it), "bad" (broken:
    /// fix it), "off" (can't tell until an earlier one passes) or "restart"
    /// (fixed, applies after signing in again). `ready` when Kiri can run
    /// containers for this account; docker and builds are offered, not required
    /// (someone may keep docker on another engine on purpose).
    pub fn checks() -> serde_json::Value {
        let installed = ["/lib/systemd/system/caskel.service", "/usr/lib/systemd/system/caskel.service", "/etc/systemd/system/caskel.service"]
            .iter()
            .any(|p| Path::new(p).exists());
        let (running, access) = match std::os::unix::net::UnixStream::connect(SOCKET) {
            Ok(_) => ("ok", "ok"),
            // The engine is there; this session may not use it. In the group
            // already but refused: the group applies at the next sign-in, and
            // the access list that would have let it in now isn't there.
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied => ("ok", if in_group_not_session() { "restart" } else { "no" }),
            Err(_) if Path::new(SOCKET).exists() => ("bad", "off"),
            Err(_) => ("no", "off"),
        };
        let usable = access == "ok";
        let docker = if !usable { "off" } else if docker_uses_kiri() { "ok" } else { "no" };
        let builds = if !usable { "off" } else if builder_made() { "ok" } else { "no" };
        let ready = installed && usable;
        // Setup is done when docker and builds pass too, or someone chose to
        // skip them (docker kept on another engine on purpose): until then
        // the checklist stays and Open Kiri waits.
        let skipped = skipped();
        let done = |k: &str, state: &str| state == "ok" || skipped.iter().any(|s| s == k);
        let complete = ready && done("docker", docker) && done("builds", builds);
        serde_json::json!({
            "type": "onboarding",
            "ready": ready,
            "complete": complete,
            "skipped": skipped,
            "user": user(),
            "checks": {"engine": if installed { "ok" } else { "no" }, "running": running, "access": access, "docker": docker, "builds": builds},
        })
    }

    /// Whether to show the checklist at all: setup isn't complete.
    pub fn needed(checks: &serde_json::Value) -> bool {
        checks["complete"] != true
    }

    /// The optional checks someone chose to skip ("docker", "builds"), kept
    /// in the window's settings (~/.kiri/ui.json, `setupSkip`).
    fn skipped() -> Vec<String> {
        crate::ui::prefs()["setupSkip"].as_array().into_iter().flatten().filter_map(|v| v.as_str().map(str::to_owned)).collect()
    }

    /// Skips one optional check, or takes the skip back (`undo`).
    pub fn skip(what: &str, undo: bool) -> serde_json::Value {
        let mut list = skipped();
        list.retain(|s| s != what);
        if !undo && matches!(what, "docker" | "builds") {
            list.push(what.to_owned());
        }
        serde_json::json!(list)
    }

    /// The inherited GTK onboarding must not make privileged changes.
    /// Native Caskel onboarding will require a separately reviewed design.
    pub fn fix(_what: &str) -> io::Result<()> {
        Err(io::Error::other(
            "Automatic Caskel setup is not supported yet. Do not change the existing Docker context or old Kiri service.",
        ))
    }

    /// Never restart services through the unfinished GTK onboarding.
    pub fn restart_engine() -> io::Result<()> {
        Err(io::Error::other(
            "Automatic service restarts are disabled in Caskel's optional GUI. Manage caskel.service explicitly.",
        ))
    }

    /// `docker` reaches Kiri: its current endpoint is Kiri's socket, or the
    /// standard socket that leads to it. Kiri's own `docker` (the package's
    /// wrapper) does unless someone chose another context.
    pub fn docker_uses_kiri() -> bool {
        let Some(docker) = crate::client::docker_cli() else { return false };
        let Ok(out) = Command::new(docker).args(["context", "inspect", "--format", "{{.Endpoints.docker.Host}}"]).stderr(Stdio::null()).output() else { return false };
        let host = String::from_utf8_lossy(&out.stdout).trim().to_owned();
        match host.strip_prefix("unix://") {
            Some(p) if p == SOCKET => true,
            Some(p) => std::fs::canonicalize(p).is_ok_and(|real| real == Path::new(SOCKET)),
            None => false,
        }
    }

    fn builder_made() -> bool {
        let dir = std::env::var_os("BUILDX_CONFIG").map(std::path::PathBuf::from).or_else(|| {
            let config = std::env::var_os("DOCKER_CONFIG").map(std::path::PathBuf::from).or_else(|| std::env::var_os("HOME").map(|h| Path::new(&h).join(".docker")))?;
            Some(config.join("buildx"))
        });
        dir.is_some_and(|d| d.join("instances").join(crate::builds::BUILDER.get()).exists())
    }

    /// In the service socket's docker group, but not in the current session.
    fn in_group_not_session() -> bool {
        let me = user();
        let listed = crate::daemon::access::members("docker").is_ok_and(|m| m.contains(&me));
        listed && !session_has_group("docker")
    }

    fn session_has_group(group: &str) -> bool {
        let Ok(name) = std::ffi::CString::new(group) else { return false };
        // SAFETY: getgrnam with a valid C string, read at once; getgroups
        // fills at most the buffer's length.
        unsafe {
            let g = libc::getgrnam(name.as_ptr());
            if g.is_null() {
                return false;
            }
            let gid = (*g).gr_gid;
            if libc::getegid() == gid {
                return true;
            }
            let mut groups = vec![0 as libc::gid_t; 256];
            let n = libc::getgroups(groups.len() as libc::c_int, groups.as_mut_ptr());
            n > 0 && groups[..n as usize].contains(&gid)
        }
    }

    fn user() -> String {
        // SAFETY: getpwuid on our own uid; the name is copied at once.
        let from_uid = unsafe {
            let p = libc::getpwuid(libc::getuid());
            (!p.is_null()).then(|| std::ffi::CStr::from_ptr((*p).pw_name).to_string_lossy().into_owned())
        };
        from_uid.or_else(|| std::env::var("USER").ok()).unwrap_or_default()
    }

    /// No host logout/reboot from the inherited container GUI.
    pub fn log_out() -> io::Result<()> {
        Err(io::Error::other("Log out using your desktop session controls; Caskel will not end a host session."))
    }

    pub fn restart() -> io::Result<()> {
        Err(io::Error::other("Reboot using your desktop session controls; Caskel will not reboot the host."))
    }

}
