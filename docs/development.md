# Developing Caskel on Linux

Caskel is an independently maintained, Linux-first Rust container engine
with a Docker-compatible API. Its headless runtime and agent use the host
Linux kernel; they do not boot a macOS guest VM. The actively tested build
target is **x86_64-linux**. The optional Linux GTK/WebKit window has not yet
been packaged or accepted as part of the minimal Nix product.

## Enter the reproducible toolchain

From a Caskel source checkout, with Nix installed:

```fish
nix develop
```

Check the locked build inputs:

```fish
nix flake check
```

Build the supported package:

```fish
nix build .#caskel
```

Inspect the output without installing it:

```fish
ls -l result/bin/caskel result/bin/caskel-init result/lib/systemd/system/caskel.service
```

Cargo package names are now Caskel-native: `caskel`, `caskel-init`,
`caskel-image`, `caskel-proto`, `caskel-share-proto`, and
`caskel-agent-transport`. The historic *source directory names*
(`crates/kiri-*`) remain for upstream patch compatibility. In particular,
`crates/kiri-vmm/src/process.rs` implements **Linux process launch and
Unix-socket transport**, not an obsolete guest VM.

## Architecture

```text
Docker-compatible client
       |
       | Docker Engine HTTP API on a private Unix socket
       v
caskel CLI / engine daemon
       |
       | Linux host agent transport (Unix sockets)
       v
caskel-init (managed Linux agent)
       |
       +-- Linux namespaces / cgroups / seccomp / overlayfs
       +-- container processes on the host kernel
```

The isolated, opt-in production-facing profile uses:
- `/nix/var/nix/profiles/caskel` for the installed Nix package;
- `caskel.service` for systemd;
- `/run/caskel/docker.sock` for the Docker-compatible API;
- `/var/lib/caskel-daemon/data` for persistent engine state.

The canonical Linux service configuration uses `CASKEL_*` variables.
Older `KIRI_*` spellings and protocol/path names remain supported as
explicit compatibility fallbacks; they are **not** the product identity.
See [branding and compatibility](branding-compatibility.md).

## Tests

For nonprivileged Rust unit tests:

```fish
nix develop --command cargo test --locked -p caskel -p caskel-proto -p caskel-agent-transport
```

Check source formatting:

```fish
nix develop --command cargo fmt --all -- --check
```

The UI i18n/page checks run in GitHub Actions. They need Python 3 and a
supported headless Chrome/Chromium on the runner; the application is Rust,
not a Python service.

The Nix GitHub workflow also runs privileged Linux agent unit tests and
disposable runtime smoke tests on an ephemeral CI VM. These checks exercise
security profiles, an actual container and restart persistence. They are
**not** proof that every Compose, volume, port, DNS, GUI or production
migration scenario works.

Do not run privileged test suites against a host containing important
containers or data. Run service cutover and storage migration tests in
disposable environments with explicit rollback first.

## Docker coexistence and migration

The existing Docker service, default context, and
`/var/run/docker.sock` remain untouched by ordinary builds. Caskel's
socket and state are independent.

The first Docker migration CLI milestone is read-only metadata inventory.
A later dry-run must report unsupported features and conflicts; actual
migration is service-by-service, with consistent backups and verified
rollback. Never share live database storage read-write between Docker and
Caskel.

See [Nix packaging](../nix/README.md),
[Arch/EndeavourOS systemd and rollback](../nix/systemd/README.md),
[upstream policy](../UPSTREAM.md),
[verification roadmap](roadmap.md), and
[Docker migration plan](linux-migration-plan.md).
