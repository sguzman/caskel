# Linux Docker Engine → Caskel migration

**Status: initial read-only inventory implemented. Planning and migration remain unimplemented.**

Upstream contains an importer at `crates/kiri/src/import.rs`, built
around Docker Desktop and OrbStack contexts. That code is not exposed by
the Linux CLI and does not accept the normal local Docker daemon as a source.
Our first Linux-specific feature will adapt it rather than risk an ad hoc
bulk migration.

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

**Planned, not yet implemented:** `caskel migrate docker --dry-run`.

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
