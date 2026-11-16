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
`checks.caskel`, and `devShells.default`. The source Cargo packages retain
upstream names internally (`kiri` and `kiri-init`) to ease upstream
integration; the Nix package installs the public `caskel` command.

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

**The native, read-only Docker inventory is implemented** as `caskel migrate docker --list [--json]`; the migration dry-run and cutover are **not** implemented. Caskel does not yet move services or storage.

## Origins and license

Caskel is derived from `libracoder/kiri` under the Apache-2.0 license.
Original copyright notices remain intact. Upstream states that its name and
logo are excluded from that license; Caskel does not claim or use them as its
own branding. This project is not endorsed by upstream.

See [UPSTREAM.md](UPSTREAM.md) for our branching/integration process,
[LICENSE](LICENSE) and [NOTICE](NOTICE). Historical release notes under `docs/releases/` document upstream Kiri and
must not be read as Caskel release announcements. Current support is covered
by `docs/features.md` and `RELEASES.md`.

## Legacy Kiri Cloud separation

Caskel is **not** Kiri Cloud. The inherited Share and diagnostics code
still knows how to contact `edge.runkiri.com`, but it refuses that upstream
endpoint unless the initiating process explicitly sets
`CASKEL_ALLOW_LEGACY_KIRI_CLOUD=1`. An old Kiri account or config file does
not provide consent to send Caskel reports, account tokens or workload
metadata. The opt-in is only for intentional compatibility with an
upstream-operated service. There is **no Caskel Cloud** service or
automatic report-upload agreement.
