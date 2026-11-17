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
| C3b | Retire AppKit UI/backend and Mac-only UI strings, preserving Linux GTK path | 2 | 2 | Verified — PR #13, with optional GTK compilation addressed separately in PR #32 |
| C4 | Audit and retire or adapt remaining unused guest kernels, platform assets and Linux packaging paths | 4 | 2 | Partial — PRs #15–#16 retired obsolete installer/scripts; #22, #28, #31 retired old brand, Mac icon and screenshot assets. Historical GPL guest-kernel corresponding source intentionally retained; remaining kernel/build path audit required |
| C5 | Rebrand internal agent/crate and runtime/public names with an explicit compatibility and rollback plan | 4 | 4 | Verified — #21, #27, #29, #30: Caskel Cargo/agent, public API, diagnostics, and canonical `CASKEL_*` service settings with documented legacy aliases and disposable CI |
| D1 | Accurate Linux-first README and documented upstream-cherry-pick policy | 2 | 2 | Verified — PR #5, `UPSTREAM.md` |
| D2 | Rewrite development, feature, security, release and contributor documentation for tested Linux behavior | 4 | 4 | Verified — PR #18, four CI workflows |
| D3 | Remove upstream product branding from current-facing UI/assets; retain legally required provenance | 4 | 4 | Verified — #22, #25, #26, #28, #31: original Caskel SVGs, localized UI/About, support/docs, retired Kiri images, source/license retained; GTK runtime QA remains optional/outstanding |
| E1 | Native, read-only Docker inventory of containers, images, networks, Compose projects and persistence | 7 | 5 | Partial — PR #17, GET-only source, populated fixture and disposable live Docker smoke; representative user-host QA remains |
| E2 | Deterministic non-mutating migration dry-run with collision, gap, storage and rollback diagnostics | 8 | 0 | Pending |
| F1 | Verified image transfer from Docker/registry into Caskel | 4 | 0 | Pending |
| F2 | Recreate supported Compose/containers/networking without hijacking Docker | 4 | 0 | Pending |
| F3 | Application-consistent persistent-data migration, ownership and restore verification | 4 | 0 | Pending |
| F4 | Per-service stop/health/cutover and reversible rollback to source Docker | 3 | 0 | Pending |
| G1 | Multi-service Linux acceptance matrix for security, Compose, DNS/ports, data retention and restart/restore | 5 | 0 | Pending |
| G2 | Pinned release and staged real-host promotion with repeatable rollback evidence | 5 | 0 | Pending — never auto-redeploy production |

**Verified score after the branding PRs #21–#31:** **58 / 100** (+9 since the previous 49/100 ledger: platform-artifact audit +1, Caskel internal/runtime name migration +4, current-facing branding +4). This remains a *weighted engineering estimate* for the whole Linux runtime and future Docker migration—not a claim that Caskel's entire functionality is 58% tested. The inventory has 2 points reserved for QA on a representative Docker host. A GTK compile check was added separately in PR #32; a real desktop GUI acceptance session remains outstanding.

## Current priorities

**Maintainer priority gate (2026-11-16): BRANDING FIRST.** Before any further
kernel, Docker migration, host regression or unrelated feature development,
complete and verify the remaining Caskel source/CLI/UI/documentation
branding, then correct the GitHub About description and inherited `runkiri.com`
homepage through the repository owner's settings. The open migration and
host-test PRs #23 and #35 are parked; do **not** merge or extend them until
the branding gate is explicitly closed. Real Linux GTK desktop acceptance
remains separately pending and must never be misreported as covered by a
compile-only CI check. Keep historical Kiri copyright attribution,
certificate/storage identities and compatibility APIs until safe migration.

**After that branding gate is closed**, resume these engineering priorities:

1. Finish legally safe disposal/audit of old guest-kernel build dependencies. Preserve required GPL corresponding-source artifacts and `crates/kiri-vmm/src/process.rs` (now the `caskel-agent-transport` package).
2. Keep canonical Caskel identity and `CASKEL_*` configuration while preserving tested historical `KIRI_*`, `/kiri/*` and DNS/TLS interfaces until safe migration. The GitHub repository's description/homepage still require an owner-level setting change outside source control.
3. Validate the read-only Docker inventory against a representative existing Docker host (no mutation), then implement the deterministic dry-run before any service transfer.
4. Extend disposable-host tests of networking, mounts, persistence and rollback before production cutover.

## Non-negotiable safety and completion gates

- `main` is the tested integration line; develop on task branches and merge reviewed PRs after applicable checks pass.
- The default Docker context and `/var/run/docker.sock` remain untouched; Caskel uses its own socket/profile.
- Do not attach production database state read-write to two live engines. Do not delete older profiles, Nix generations or backup copies as cleanup.
- CI uses disposable Linux runners. An optional GTK window is not claimed tested until built and exercised as a Linux GUI.
- 100% requires all scored deliverables and all stated acceptance gates, not merely a green compile or empty PR list.

**References:** [Linux-only audit](linux-only-audit.md), [Linux Docker migration plan](linux-migration-plan.md), [upstream policy](../UPSTREAM.md), [Nix/systemd setup](../nix/systemd/README.md). Update this ledger on substantive verified milestones; do not append chat transcripts or routine progress noise.
