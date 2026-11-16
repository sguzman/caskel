//! Caskel's Linux-native command-line and daemon entry point.

#[cfg(not(target_os = "linux"))]
compile_error!("Caskel supports Linux hosts only");

use std::process::ExitCode;

// builds, cli_pins, scan and tools on Linux too: the daemon reads an
// image's layers and scans it for kiri-win, whose window can't reach the
// store inside WSL (daemon/api.rs).
#[cfg(unix)]
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
mod builds;
// The engine stops when its app is deleted under it (kiri-win includes it
// for Windows).
#[cfg(unix)]
mod appwatch;
// Which builder plain `docker build` uses: Docker's or Kiri's.
#[cfg(unix)]
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
mod buildwith;
// Linux only, but its tests run anywhere with unix sockets.
#[cfg(any(target_os = "linux", all(test, unix)))]
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
mod bridge;
// Linux only (WSL's end of kiri-win's proxy relay); its tests run anywhere.
#[cfg(unix)]
mod hostrelay;
#[cfg(unix)]
mod cli_pins;
#[cfg(unix)]
mod client;
#[cfg(unix)]
mod daemon;
#[cfg(unix)]
mod diag;
#[cfg(any(unix, windows))]
mod docker;
mod flavor;
mod images;
#[cfg(unix)]
mod guest;
#[cfg(target_os = "linux")]
mod hostaccess;
#[cfg(target_os = "linux")]
mod migrate;
#[cfg(unix)]
mod k8s;
// kubectl's own file, which `kiri k8s` puts its context in.
#[cfg(unix)]
mod kubeconfig;
#[cfg(unix)]
mod machine;
#[cfg(unix)]
mod run;
#[cfg(unix)]
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
mod scan;
#[cfg(unix)]
mod usb;
#[cfg(unix)]
mod stale;
#[cfg(any(unix, windows))]
mod share;
#[cfg(all(target_os = "linux", feature = "window"))]
mod ui;
#[cfg(unix)]
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
mod tools;

const USAGE: &str = "\
usage: caskel COMMAND          docker's commands, on Caskel: caskel run, ps, exec,
                             logs, stop, rm, images, pull, build, compose…
       caskel daemon
       caskel bridge [--socket PATH]
       caskel migrate docker --list [--socket PATH] [--json]   read-only Docker inventory
       caskel ui [--start] [--view containers|images|volumes|networks]
       caskel build-with [docker|caskel]   which builder docker build uses
       caskel diag logs [-f] [--ui|--engine|--cli] [--errors] [-n N]
       caskel diag report [--out PATH] [--send]   (a file to send us, tokens taken out)
       caskel PORT [NAME]           share a local port on the internet (same as caskel share PORT)
       caskel share PORT|CONTAINER [--name N] | share login | logout | status
       caskel k8s start [--use] | stop | status | delete | set [--lan on|off] [--tls-san NAME,…] [--port N]
       caskel usb [list] | attach DEVICE | detach DEVICE   USB devices for containers, beta
       caskel machine create NAME [--distro D] | list | shell NAME | stop | delete NAME
       caskel raw run [--rm] [-e KEY=VALUE]... IMAGE [COMMAND...]   (its own guest, no daemon)
       caskel raw pull <image> | raw images | raw rmi <image>...";

fn main() -> ExitCode {
    flavor::init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    // Which process this is, for its log and crash reports.
    #[cfg(unix)]
    diag::init(match args.first().map(String::as_str) {
        Some("daemon") => diag::Part::Daemon,
        // Only a build with the window: elsewhere `kiri ui` is a plain
        // command whose message must reach the terminal, not the log's pipe.
        #[cfg(all(target_os = "linux", feature = "window"))]
        Some("ui") => diag::Part::Ui,
        _ => diag::Part::Cli,
    });
    let result: std::io::Result<()> = match args.first().map(String::as_str) {
        #[cfg(unix)]
        Some("daemon") => daemon::run(&args[1..]),
        #[cfg(target_os = "linux")]
        Some("bridge") => bridge::run(&args[1..]),
        #[cfg(target_os = "linux")]
        Some("hostrelay") => hostrelay::run(),
        // Linux hosts: what the window's onboarding needs root for (through pkexec).
        #[cfg(target_os = "linux")]
        Some("host-access") => match hostaccess::run(&args[1..]) {
            Ok(code) => return ExitCode::from(code),
            Err(e) => Err(e),
        },
        #[cfg(target_os = "linux")]
        Some("migrate") => migrate::run(&args[1..]),
        #[cfg(unix)]
        Some("raw") => match args.get(1).map(String::as_str) {
            Some("run") => run::run(&args[2..]),
            Some("pull") => images::pull(&args[2..]),
            Some("images") => images::list(&args[2..]),
            Some("rmi") => images::remove(&args[2..]),
            _ => {
                eprintln!("{USAGE}");
                return ExitCode::FAILURE;
            }
        },
        #[cfg(any(unix, windows))]
        Some("share") => share::run(&args[1..]),
        // `kiri 3000`: share a port straight away.
        #[cfg(any(unix, windows))]
        Some(a) if share::is_shorthand(a) => share::run(&args),
        #[cfg(unix)]
        Some("build-with") => buildwith::cli(&args[1..]),
        #[cfg(unix)]
        Some("k8s") => k8s::run(&args[1..]),
        #[cfg(unix)]
        Some("machine") => machine::run(&args[1..]),
        #[cfg(unix)]
        Some("usb") => usb::run(&args[1..]),
        #[cfg(unix)]
        Some("diag") => diag::run(&args[1..]),
        #[cfg(all(target_os = "linux", feature = "window"))]
        Some("ui") => ui::run(
            args.iter().any(|a| a == "--start"),
            args.iter().position(|a| a == "--view").and_then(|i| args.get(i + 1)).cloned(),
        ),
        // The static engine has no window: the package's own build of it.
        #[cfg(all(target_os = "linux", not(feature = "window")))]
        Some("ui") => window(&args),
        #[cfg(any(unix, windows))]
        Some(c) if docker::handles(c) => {
            engine_version_check();
            docker::run(&args)
        }
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::FAILURE;
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            // The subcommand only: arguments can carry values.
            #[cfg(unix)]
            if !e.to_string().starts_with("usage:") {
                // Only the subcommand is logged: args may contain private values.
                diag::warn!("command failed", command = args.first().map_or(Some("open"), |a| (a.len() <= 20 && a.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')).then_some(a.as_str())).unwrap_or(""), error = e);
            }
            eprintln!("caskel: {e}");
            ExitCode::FAILURE
        }
    }
}

/// An optional, separately packaged Linux GTK/WebKit window for Caskel.
/// The headless Nix package does not install it.
#[cfg(all(target_os = "linux", not(feature = "window")))]
const WINDOW: &str = "/usr/lib/caskel/caskel-window";

/// `caskel ui` on Linux: launch the optional window if installed.
#[cfg(all(target_os = "linux", not(feature = "window")))]
fn window(args: &[String]) -> std::io::Result<()> {
    use std::os::unix::process::CommandExt;
    if !std::path::Path::new(WINDOW).exists() {
        return Err(std::io::Error::other(
            "this Caskel build has no Linux window; install a window-enabled Caskel package or build with the 'window' feature",
        ));
    }
    // exec returns only when it failed.
    Err(std::process::Command::new(WINDOW).args(args).exec())
}

#[cfg(not(unix))]
fn engine_version_check() {}

/// Before a Docker-compatible command, warn if the CLI and daemon versions
/// differ (on stderr, so machine-readable output remains clean).
#[cfg(unix)]
fn engine_version_check() {
    let me = env!("CARGO_PKG_VERSION");
    if me == "0.0.0" || !client::alive() {
        return;
    }
    let Some(engine) = client::request("GET", "/version", None).ok().filter(|r| r.ok()).and_then(|r| client::kiri_version(&r.json()).map(str::to_owned)) else { return };
    if engine != me && engine != "0.0.0" {
        let exe = std::env::current_exe().map(|p| p.display().to_string()).unwrap_or_else(|_| "caskel".into());
        eprintln!("caskel: this CLI ({exe}) is {me}, but the engine runs {engine}. Restart the Caskel daemon to use one version.");
    }
}
