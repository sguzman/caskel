# Caskel

![Caskel original logo](brand/caskel-wordmark.svg)

**Linux-first container runtime, written in Rust.**

Caskel is a Linux-focused, independently maintained fork of
[Kiri](https://github.com/libracoder/kiri). It speaks the Docker Engine API,
so familiar Docker and Compose commands can drive the new engine without
replacing the existing Docker installation.

## Goals

- Linux is the primary product, rather than a secondary desktop-app target.
- Build and package reproducibly with Nix, including an opt-in systemd service.
- Dogfood real Linux workloads and promote changes only after testing.
- Build a safe native Docker-to-Caskel migration workflow, starting with a
  read-only inventory and dry-run planner, then reversible per-service cutovers.
- Review upstream bug fixes, security improvements, performance work and
  features **selectively**; our `main` is maintained independently.

## Build (x86_64 Linux)

```fish
nix build .#caskel
```

```fish
./result/bin/caskel --help
```

The flake exposes `packages.caskel`, `packages.default`, `apps.default`,
`checks.caskel`, and `devShells.default`. The Cargo packages are now Caskel-named (`caskel`, `caskel-init`, and
supporting `caskel-*` crates). Only compatibility-sensitive protocol fields,
legacy runtime paths and the `kiri-init` executable symlink retain older names.

**Building alone does not install or start a privileged service.** Follow
[the Arch / EndeavourOS installation guide](nix/systemd/README.md) for a
dedicated root-owned Nix profile, isolated systemd unit, smoke test and rollback.

## Commands and coexistence

| Command | Engine |
|---|---|
| `docker ps` | Existing production Docker Engine |
| `caskel ps` | Caskel |
| `caskel images` | Caskel image inventory |
| `caskel run --rm --network none alpine:3.21 echo hello` | Caskel |
| `docker -H unix:///run/caskel/docker.sock ps` | Caskel explicitly |

The dedicated Caskel engine uses `/run/caskel/docker.sock`,
`/var/lib/caskel-daemon/data`, and `caskel.service`. It does **not**
replace Docker's socket or set the default Docker context. Its images and
containers are independent of Docker and the earlier Kiri installation.

**Do not run the original Kiri daemon and Caskel simultaneously** during
the transition. Their internal network management shares names. Docker may
remain running, but host ports, bridges and firewall changes must be tested.

The initial Nix build is headless on Linux (CLI and runtime agent).

## Roadmap

The [verified development roadmap](docs/roadmap.md) tracks full-lifecycle milestones, evidence, and remaining acceptance gates.

1. Independent project identity, reproducible Nix build and rollback-safe service.
2. [Docker-to-Caskel migration tooling](docs/linux-migration-plan.md),
   with read-only inventory and dry-run first.
3. Linux integration tests for Compose, networking, volumes and persistence.
4. Selective upstream synchronization and tracked security updates.
5. Gradual, recoverable migration of self-hosted workloads.

**Read-only Docker inventory and a conservative migration preview are implemented** as `caskel migrate docker --list [--json]` and `caskel migrate docker --dry-run [--json]`. The preview reports prerequisites and potential source-side conflicts, but does not inspect target readiness or approve any workload for cutover. Image transfer, persistent-data migration and service cutover remain unimplemented.

To **optionally** inspect the separate Caskel target's configured host ports
without changing either engine, use
`caskel migrate docker --dry-run --target-socket /run/caskel/docker.sock`.
This reads the target's container and network metadata using GET requests only
and flags *potential* cross-engine host-port and IPv4/IPv6 subnet overlap.
Without this explicit flag, the target is not inspected. An independent
`--inspect-host-listeners` opt-in adds a read-only Linux `/proc/net`
snapshot of TCP/UDP sockets matching configured Docker source ports.
It reads only the CLI's own network namespace; it cannot identify port owners,
inspect other namespaces, establish future availability, or authorize a cutover.
The independent `--inspect-host-storage` opt-in reads only metadata of
Docker source bind/volume mount paths (existence, type, uid/gid, mode, and
available filesystem bytes), without reading contents or following the final
symlink. It observes the CLI's own mount namespace, not necessarily Docker's
namespace, and **does not validate destination storage, backup or restore**.
All three opt-in flags can be combined with `--dry-run --json`.
Treat the JSON output as private infrastructure metadata. Caskel does not move or stop existing Docker services.

## Origins and license

Caskel is derived from `libracoder/kiri` under the Apache-2.0 license.
Original copyright notices remain intact. Upstream states that its name and
logo are excluded from that license; Caskel does not claim or use them as its
own branding. This project is not endorsed by upstream.

See [UPSTREAM.md](UPSTREAM.md) for our branching/integration process,
[LICENSE](LICENSE) and [NOTICE](NOTICE). Historical release notes under `docs/releases/` document upstream Kiri and
must not be read as Caskel release announcements. Current support is covered
by `docs/features.md` and `RELEASES.md`.

## Local-only diagnostics and no cloud sharing

Caskel has **no cloud account, hosted sharing, or diagnostics upload service**.
Local logs and redacted diagnostic report export remain available. Legacy Kiri
Cloud and Share network connections are rejected at the transport boundary,
even if an old opt-in environment variable, account token, or relay setting
is present. The `caskel share` command is unsupported. Docker image pulls
and other explicitly requested registry operations are separate functionality.

## Runtime configuration identity

On Linux, prefer `CASKEL_SOCKET`, `CASKEL_DATA_DIR`,
`CASKEL_AGENT`, `CASKEL_SOCKET_GROUP`, `CASKEL_HTTP_PORT`, and
`CASKEL_HTTPS_PORT`. The older `KIRI_*` spellings still work as fallbacks
for pinned legacy units and test configurations; when both spellings are
set, the Caskel form takes precedence. The shipped `caskel.service` uses
only the canonical Caskel variables. Updating source does **not** replace a
locally deployed systemd unit or restart an engine.
