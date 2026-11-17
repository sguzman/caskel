# Caskel: implemented and verified features

Caskel is a **Linux-native, headless-first** container engine written in
Rust. It speaks much of the Docker Engine API, so familiar Docker and
Compose clients can address Caskel explicitly without replacing Docker.

This page distinguishes *implemented code* from *verified acceptance*.
Features in the inherited codebase are not automatically certified merely
because an upstream document claimed support.

## Supported development baseline

- Host platform: **x86_64 Linux**, tested with a pinned Nix flake.
- Build output: `caskel` CLI/daemon, `caskel-init` Linux agent,
  optional `caskel.service` unit.
- Runtime: Linux process transport, namespaces, cgroups, filesystem layers,
  container processes, and a private Unix-socket Docker-compatible API.
- Isolation: a dedicated Caskel Nix profile, runtime socket and data store;
  no takeover of Docker's default context or service.
- Security: Docker-style default/custom seccomp policies and agent
  lifecycle safeguards, selectively integrated from upstream.
- Reliability: cleanup/recovery of stale container roots after daemon
  restarts; a disposable CI container verifies restart persistence.

The core acceptance workflow builds the Nix package, runs Linux Rust tests
(including privileged agent tests), and starts disposable containers on a
CI VM. CodeQL, UI-page/i18n and performance workflows run independently.

## Present in the inherited engine, still requiring targeted Linux QA

The source contains Docker-compatible paths for image pulls/builds,
container creation/exec/logs, port publishing, bridges/DNS, named volumes,
bind mounts, Compose and other management commands. Do **not** interpret
this as a guarantee of compatibility for every Docker API version or
every production workload.

A full acceptance matrix is still required for real Compose projects,
published-port conflicts, DNS, network reuse, UID/GID ownership, mounts,
persistent databases, restart rules, failures and restore behavior.

## Optional Linux interface

The GTK/WebKitGTK window source remains in the repository, but the minimal
Nix package is intentionally **headless**. Current Linux UI artwork,
translations and About branding are Caskel-native, but the optional
GTK feature still needs separate build and real-desktop QA before release.
Historical Kiri protocol paths remain compatible by design.

## Docker-to-Caskel migration

The migration roadmap starts with a **read-only Docker inventory**, then a
non-mutating plan and finally reversible cutovers, one service at a time.
An inventory does not transfer images or persistent data. Nothing is
migrated or stopped automatically.

Follow [the current roadmap](roadmap.md) for the precise verified score
and [the migration plan](linux-migration-plan.md) for safety requirements.

## Not supported as Caskel product targets

The old Kiri Mac app, Apple Virtualization.framework guest, Windows/WSL
launcher, Homebrew/DMG/MSI packaging, and legacy Kiri `.deb`/`.rpm`
installer scripts are **not** Caskel distributions. The former Linux
packagers could affect `/run/docker.sock` and were deliberately retired.

The only supported packaging path so far is the
[pinned Nix build](../nix/README.md) and opt-in Linux service.
