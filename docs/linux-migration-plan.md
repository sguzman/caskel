# Linux Docker Engine → Caskel migration

**Status: read-only inventory and conservative dry-run preview implemented. Image/data transfer and cutovers remain unimplemented.**

Upstream contains an importer at `crates/kiri/src/import.rs`, built
around Docker Desktop and OrbStack contexts. That code is not exposed by
the Linux CLI and does not accept the normal local Docker daemon as a source.
Caskel's independent Linux implementation instead uses a GET-only Docker
inventory and a deterministic **blocked** preview. It does not invoke the
upstream importer or perform bulk migration.

## Milestones

1. **Read-only discovery** of images, containers, ports, labels, Compose
   projects, volumes, bind mounts, networks, dependencies and restart rules.
2. **Dry-run plan** showing what can move, what is unsupported, port/network
   conflicts and exact proposed actions. Never modify Docker in this phase.
3. **Image transfer** using Docker save/load or a verified registry pull.
4. **Container/network recreation** from Compose or inspect metadata, not
   a blind copy of Docker's internal container directories.
5. **Persistent data migration** with ownership/permissions checks and
   application-consistent backups; require a controlled stop or snapshot
   for actively written database data.
6. **Service-by-service cutover** with health verification and reversible
   rollback to the existing Docker deployment.
7. **Regression tests** for Compose, DNS, published ports, bind mounts,
   volumes, restart policies, security and state persistence.

Read-only discovery command:

```fish
caskel migrate docker --list
```

```fish
caskel migrate docker --list --json
```

The initial implementation connects directly to `/var/run/docker.sock`, or an
explicit `--socket PATH`, and issues GET requests only. It reads container,
image, network and volume metadata, including Compose project labels, mounts,
published ports and restart policies. Arbitrary label values and container
environment variables are excluded. It rejects source sockets resolving to
Caskel or the old Kiri engine, and does not change Docker's default context.

The JSON inventory contains infrastructure names and host paths: keep it
private. This is discovery only, not an application dependency graph.

Conservative, read-only cutover preview:

```fish
caskel migrate docker --dry-run
```

```fish
caskel migrate docker --dry-run --json
```

The preview uses the same GET-only discovery path as `--list`. It annotates
each workload with migration prerequisites: image availability/transfer,
backup and restore, bind/volume ownership, security/device capabilities,
custom network/DNS requirements, Compose dependencies, restart/health
checks, and host-port rebinds. It flags *potential source-side* published
port overlap, including stopped containers with configured bindings.

**This is NOT a migration action, deployable execution plan, or compatibility
certificate.** All workloads report `blocked_pending_manual_validation`.
By default the target Caskel engine is **not** inspected. When explicitly
requested with `--target-socket`, only its container port configuration and
network subnet metadata are compared with Docker. This is not a live
host-routing or socket-occupancy check. Image/platform capability,
gateway/DNS compatibility, missing or malformed IPAM data,
application-consistent data and rollback readiness have **not** been verified. The JSON lists
proposed manual steps but cannot be applied; no `--apply` flag exists.
Treat inventory and preview outputs as private infrastructure metadata.

Optional target-side read-only comparison (PRs #48–#49):

```fish
caskel migrate docker --dry-run --target-socket /run/caskel/docker.sock --json
```

The target must identify as Caskel and be a different Unix socket from
Docker's. The command uses GET only, reports potential overlapping published
host ports and IPAM IPv4/IPv6 CIDR ranges, and keeps every workload blocked.
Network-name matches do not imply safe equivalence. Invalid/unavailable
subnets cannot be treated as compatible. Never publish the JSON report.

**Verification on merged PR #23:** all four CI workflows passed, including
the disposable Linux host's real-Docker read-only inventory and JSON
preview checks. The suite rejects `--apply` and confirms the original Docker
container/image inventory is unchanged. These checks are **not** proof
of behavior against the user's production Docker host.

Still planned: validate a representative source host without mutation;
continue isolated Caskel destination checks for
network gateways, DNS, actual host port occupancy, storage
ownership/space, image/platform support and restoration
readiness; and only later design a separately authorized, reversible
per-service cutover mechanism. No source or target is modified by a preview.

## Safety principles

- Keep production Docker as the source of truth during evaluation.
- No bulk removal, container stopping, daemon replacement, or default
  Docker context switching.
- Never mount a live database directory read/write from both engines.
- Track port bindings and network address ranges before any cutover.
- Verify backups and restores on disposable workloads first.
- Preserve the original Docker Compose files and configuration as the
  rollback definition.

Code starting point: [upstream importer](../crates/kiri/src/import.rs).
