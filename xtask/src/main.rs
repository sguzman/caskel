//! Linux-only Caskel developer build tasks.
//!
//! The supported, reproducible build is the pinned Nix flake. Unlike the
//! upstream xtask, these commands do not download toolchains, run Docker,
//! package macOS apps, or produce Windows/WSL installers.

use std::path::Path;
use std::process::{Command, ExitCode};

fn usage() {
    eprintln!("usage: cargo xtask <linux|check>");
    eprintln!("  linux  build Caskel and its runtime agent with Nix");
    eprintln!("  check  evaluate and build the pinned Nix flake checks");
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(task) = args.next() else {
        usage();
        return ExitCode::FAILURE;
    };
    if args.next().is_some() {
        usage();
        return ExitCode::FAILURE;
    }

    let mut command = Command::new("nix");
    match task.as_str() {
        "linux" => { command.args(["build", ".#caskel"]); }
        "check" => { command.args(["flake", "check", "--show-trace"]); }
        _ => {
            usage();
            return ExitCode::FAILURE;
        }
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask is inside the repository");
    match command.current_dir(root).status() {
        Ok(status) if status.success() => ExitCode::SUCCESS,
        Ok(status) => {
            eprintln!("xtask {task}: nix exited with {status}");
            ExitCode::FAILURE
        }
        Err(e) => {
            eprintln!("xtask {task}: could not launch Nix: {e}");
            ExitCode::FAILURE
        }
    }
}
