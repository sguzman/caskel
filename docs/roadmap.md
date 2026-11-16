# Caskel verification roadmap

**Scope:** a Linux-first Caskel engine that can safely coexist with an existing Docker installation, discover and plan migration of selected workloads, and execute reversible service-by-service cutovers. The Linux GTK window is optional; the headless runtime is the first release target. Preserve upstream Apache-2.0 attribution and the existing Docker/Kiri installations until separately authorized host cutovers.

This is an evidence ledger, not a promise that a feature is complete just because it compiles. Work is **verified** only when its acceptance condition is met on a committed change. **Active** work scores zero until verified. Percentages are a *weighted engineering estimate*, not elapsed time, velocity, or a claim that all remaining risks are known. Recalibrate transparently if required scope changes.

## Weighted milestones (100 points)

| ID | Milestone and acceptance condition | Points | Verified | State / evidence |
| --- | --- | ---: | ---: | --- |
| A1 | Independent Linux `caskel` CLI, Nix build and systemd identity; no replacement of Docker's default socket | 7 | 7 | Verified — PR #5, Nix CI |
| A2 | Opt-in dedicated profile, data isolation, documented rollback and prior Kiri retained | 4 | 4 | Verified — PRs #3–#5, installation guide |
| A3 | Linux build, Rust/agent tests and disposable privileged container smoke wired into CI | 4 | 4 | Verified — workflow `.github/workflows/nix.yml` |
| B1 | Agent/daemon restart recovery avoids stale mounts and handles preserved containers | 5 | 5 | Verified — PR #6 and disposable CI |
| B2 | Docker-style default/custom seccomp policy, Linux lifecycle safeguards | 5 | 5 | Verified — PR #7 and disposable CI |
| B3 | Broader privileged host regressions for DNS, published ports, binds, volumes, restart and failures | 5 | 0 | Pending — distinct from current limited CI smoke |
| C1 | Retire Windows/WSL and Mac release tooling, unsupported CLI/Apple VM backend | 4 | 4 | Verified — PRs #9 and #10 |
| C2 | Prune Windows workspace, Apple dependencies, stale platform translations while retaining Linux runtime | 4 | 4 | Verified — PR #11 |
| C3a | Remove unreachable Mac guest boot code and narrow Linux agent launch config | 2 | 2 | Verified — PR #12; all four checks passed |
| C3b | Retire AppKit UI/backend and Mac-only UI strings, preserving Linux GTK path | 2 | 0 | Active — PR #13; score only after complete CI |
| C4 | Audit and retire or adapt remaining unused guest kernels, platform assets and Linux packaging paths | 4 | 0 | Pending — validate dependencies before deletion |
| C5 | Rebrand internal agent/crate and runtime/public names with an explicit compatibility and rollback plan | 4 | 0 | Pending — preserve `KIRI_*` aliases until assessed |
| D1 | Accurate Linux-first README and documented upstream-cherry-pick policy | 2 | 2 | Verified — PR #5, `UPSTREAM.md` |
| D2 | Rewrite development, feature, security, release and contributor documentation for tested Linux behavior | 4 | 0 | Pending |
| D3 | Remove upstream product branding from current-facing UI/assets; retain legally required provenance | 4 | 0 | Pending |
| E1 | Native, read-only Docker inventory of containers, images, networks, Compose projects and persistence | 7 | 0 | Pending |
| E2 | Deterministic non-mutating migration dry-run with collision, gap, storage and rollback diagnostics | 8 | 0 | Pending |
| F1 | Verified image transfer from Docker/registry into Caskel | 4 | 0 | Pending |
| F2 | Recreate supported Compose/containers/networking without hijacking Docker | 4 | 0 | Pending |
| F3 | Application-consistent persistent-data migration, ownership and restore verification | 4 | 0 | Pending |
| F4 | Per-service stop/health/cutover and reversible rollback to source Docker | 3 | 0 | Pending |
| G1 | Multi-service Linux acceptance matrix for security, Compose, DNS/ports, data retention and restart/restore | 5 | 0 | Pending |
| G2 | Pinned release and staged real-host promotion with repeatable rollback evidence | 5 | 0 | Pending — never auto-redeploy production |

**Verified score at PR #12:** 37 / 100. **Before this continuation (PR #11):** 35 / 100. PR #13 will add 2 only if its CI completes successfully and its code is merged. Host GUI compilation/QA is not implied by a passing HTML page test.

## Current priorities

1. Finish PR #13 and establish its exact test evidence; do not silently count pending runs.
2. Complete source-led platform artifact/kernel audit; keep `kiri-vmm/src/process.rs`, which is the Linux agent transport, regardless of its historic name.
3. Map all old/new config, socket, storage, package and protocol consumers before internal rebranding. Avoid breaking installed Caskel or the old Kiri rollback copy.
4. Build read-only Docker discovery before implementing any write path. Then implement dry-run planning and reversible cutovers.
5. Expand disposable-host acceptance to realistic networking and persistent volumes before migration of actual services.

## Non-negotiable safety and completion gates

- `main` is the tested integration line; develop on task branches and merge reviewed PRs after applicable checks pass.
- The default Docker context and `/var/run/docker.sock` remain untouched; Caskel uses its own socket/profile.
- Do not attach production database state read-write to two live engines. Do not delete older profiles, Nix generations or backup copies as cleanup.
- CI uses disposable Linux runners. An optional GTK window is not claimed tested until built and exercised as a Linux GUI.
- 100% requires all scored deliverables and all stated acceptance gates, not merely a green compile or empty PR list.

**References:** [Linux-only audit](linux-only-audit.md), [Linux Docker migration plan](linux-migration-plan.md), [upstream policy](../UPSTREAM.md), [Nix/systemd setup](../nix/systemd/README.md). Update this ledger on substantive verified milestones; do not append chat transcripts or routine progress noise.
