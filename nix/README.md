# Caskel Nix packaging (Linux-first)

This directory packages the Rust engine and its agent as **Caskel** on
`x86_64-linux`, using a pinned Nix flake. It does not use Docker to
build the engine, change Docker's default socket, start a privileged daemon,
or write system configuration during the build. The Linux-only
`cargo xtask linux` now delegates to `nix build .#caskel` rather than
using upstream's Docker-based cross-platform build pipeline.

## Build

From the root of this repository on EndeavourOS / Arch, using fish:

```fish
nix build .#caskel
```

```fish
ls -lh result/bin/caskel result/bin/kiri-init result/lib/systemd/system/caskel.service
```

```fish
nix flake check
```

```fish
nix develop
```

The lockfile is committed. The public `caskel` binary is built from
the original Cargo package `kiri`; `kiri-init` remains the internal
runtime agent to ease integration of upstream changes.

The optional Linux GUI is not packaged in this minimal Nix build.
Privileged runtime tests cannot run in the Nix build sandbox. GitHub Actions
checks the reproducible package and unit file, runs unprivileged Rust unit
tests, then runs privileged agent unit tests and isolated container/security
smoke tests on a disposable Linux CI VM. This does not replace local
acceptance testing of real networking, Compose or production service data.

## Installation and opt-in service

See [the safe installation guide](systemd/README.md). It uses a
dedicated `/nix/var/nix/profiles/caskel` system profile, a distinct
`caskel.service`, `/run/caskel/docker.sock`, and a new persistent
data directory. It does not replace the existing Kiri experimental service
or your running Docker services.

**Do not run both the original Kiri daemon and Caskel simultaneously.**
Do not migrate old container data by copying live daemon internals.

## Maintenance

- `main`: tested, Linux-first Caskel.
- `feat/*`, `fix/*`: fork features and fixes via PRs.
- `sync/*`: selected upstream changes, with review and tests.
- No force-push of upstream over our `main`.

Read [the upstream policy](../UPSTREAM.md) and
[the Linux Docker migration plan](../docs/linux-migration-plan.md).
The Apache-2.0 copyright notices remain intact, and Kiri's name/logo
are not adopted as Caskel branding.

Upgrade the locked `nixpkgs` input in a dedicated PR; update
`Cargo.lock` only when Rust dependencies require it. Do not automate
changes to a running root-owned service during build or install.
