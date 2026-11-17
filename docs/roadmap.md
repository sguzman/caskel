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
| B3 | Broader privileged host regressions for DNS, published ports, binds, volumes, restart and failures | 5 | 5 | Verified — disposable privileged Linux CI: persistent named volumes and read-only bind mounts (#46); private bridge and name resolution (#50); TCP publish/collision/restart/release (#52); DNS isolation, occupied-bridge refusal and stale-name cleanup (#53); DNS after daemon restart (#54); external TCP listener collision/reuse (#55); UDP host collision, datagram reply and reuse (#56); real Docker-published port coexistence and release (#57). All four CI workflows passed for #57. Broader multi-service acceptance and migration rollback remain separate G1/F4 gates |
| C1 | Retire Windows/WSL and Mac release tooling, unsupported CLI/Apple VM backend | 4 | 4 | Verified — PRs #9 and #10 |
| C2 | Prune Windows workspace, Apple dependencies, stale platform translations while retaining Linux runtime | 4 | 4 | Verified — PR #11 |
| C3a | Remove unreachable Mac guest boot code and narrow Linux agent launch config | 2 | 2 | Verified — PR #12; all four checks passed |
| C3b | Retire AppKit UI/backend and Mac-only UI strings, preserving Linux GTK path | 2 | 2 | Verified — PR #13, with optional GTK compilation addressed separately in PR #32 |
| C4 | Audit and retire or adapt remaining unused guest kernels, platform assets and Linux packaging paths | 4 | 3 | Partial — prior installer/artwork cleanup (#15–#16, #22, #28, #31); Nix excludes the historical guest-kernel tree (#44, verified CI), upstream release-source metadata is inventoried, and old GPL material remains preserved. Independent archive correspondence and legal review remain open |
| C5 | Rebrand internal agent/crate and runtime/public names with an explicit compatibility and rollback plan | 4 | 4 | Verified — #21, #27, #29, #30: Caskel Cargo/agent, public API, diagnostics, and canonical `CASKEL_*` service settings with documented legacy aliases and disposable CI |
| D1 | Accurate Linux-first README and documented upstream-cherry-pick policy | 2 | 2 | Verified — PR #5, `UPSTREAM.md` |
| D2 | Rewrite development, feature, security, release and contributor documentation for tested Linux behavior | 4 | 4 | Verified — PR #18, four CI workflows |
| D3 | Remove upstream product branding from current-facing UI/assets; retain legally required provenance | 4 | 4 | Verified — #22, #25, #26, #28, #31; native Hyprland GTK branding QA recorded via #43, with specified translation/Network-pane/post-#42 visual limits |
| E1 | Native, read-only Docker inventory of containers, images, networks, Compose projects and persistence | 7 | 5 | Partial — PR #17, GET-only source, populated fixture and disposable live Docker smoke; representative user-host QA remains |
| E2 | Deterministic non-mutating migration dry-run with collision, gap, storage and rollback diagnostics | 8 | 0 | Active — PR #23 merged: GET-only source inventory → deterministic, blocked dry-run; source-side port reuse and storage/security/Compose/rollback prerequisites tested in unit + disposable CI. Opt-in target-port (#48) and IPv4/IPv6 subnet-overlap (#49) warnings verified by four CI workflows each; actual host occupancy, DNS/IPAM semantics, restore readiness and full actionable compatibility diagnosis remain unverified; no readiness or cutover claims |
| F1 | Verified image transfer from Docker/registry into Caskel | 4 | 0 | Pending |
| F2 | Recreate supported Compose/containers/networking without hijacking Docker | 4 | 0 | Pending |
| F3 | Application-consistent persistent-data migration, ownership and restore verification | 4 | 0 | Pending |
| F4 | Per-service stop/health/cutover and reversible rollback to source Docker | 3 | 0 | Pending |
| G1 | Multi-service Linux acceptance matrix for security, Compose, DNS/ports, data retention and restart/restore | 5 | 0 | Pending |
| G2 | Pinned release and staged real-host promotion with repeatable rollback evidence | 5 | 0 | Pending — never auto-redeploy production |

**Verified score after the B3 privileged Linux regression matrix:** **64 / 100** (+5 from 59/100, after merged PRs #46 and #50–#57 passed their applicable disposable-host CI gates). The preceding **59 / 100** baseline followed #44's C4 build-input isolation/provenance ledger; the final GPL/release correspondence point is still open. The earlier branding baseline was **58 / 100**. This remains a *weighted engineering estimate* for the whole Linux runtime and future Docker migration—not a claim that Caskel's entire functionality is 64% tested. E1 still reserves 2 points for representative Docker-host QA; E2, F1–F4 and G1–G2 are not awarded by B3's isolated regressions. Native GTK visual branding acceptance was recorded in PR #43 with its specific limits; production migration readiness is not implied.

## Current priorities

**Branding-first gate: ACCEPTED on 2026-11-17, with documented limits.**
PRs #40–#42 removed the inherited cloud feature from Linux, made VM-style
resource controls read-only, and suppressed the startup Share request. All
required CI workflows passed. The owner exercised the Linux GTK GUI on
Hyprland: Caskel identity, dashboard, local Diagnostics, read-only Resources,
Dark/Light/System themes and Spanish switching. Translation quality in all 37
languages, RTL/CJK desktop visuals, Linux Network-pane visuals and the last
startup-toast fix were not individually manually retested. See the
[desktop acceptance record](gtk-desktop-acceptance.md#real-desktop-acceptance-record--2026-11-17).
The recorded limitations do **not** justify additional compulsory user
localization QA. The branding QA itself left the score unchanged. The separately verified
C4 build-input isolation and source-metadata audit contributed +1 to the earlier 59/100 baseline;
the final GPL provenance point is still open.

The read-only migration preview in PR #23 is **merged**, followed by
opt-in, GET-only target port inspection (#48) and subnet-overlap warnings
(#49), both merged after four green CI checks. It never applies a plan
or approves a cutover. The outdated storage PR #35 was **closed without merge**,
replaced by merged PR #46's disposable-host persistence and read-only-bind
tests. The B3 regression sequence #50–#57 now covers isolated DNS,
ports (including UDP), restart recovery, collision refusal, and real Docker
published-port coexistence; #57 passed all four workflows including the
privileged Linux runtime smoke. All these tests run on disposable CI hosts.
Neither implies production readiness or permits changes to production Docker state.

**Now that branding acceptance is recorded**, follow these engineering priorities:

1. Finish legally safe disposal/audit of old guest-kernel build dependencies. Preserve required GPL corresponding-source artifacts and `crates/kiri-vmm/src/process.rs` (now the `caskel-agent-transport` package).
2. Keep canonical Caskel identity and `CASKEL_*` configuration while preserving tested historical `KIRI_*`, `/kiri/*` and DNS/TLS interfaces until safe migration. The GitHub repository About description/homepage were corrected by the owner on 2026-11-17.
3. Validate existing GET-only Docker inventory and the blocked dry-run against a representative source host **without mutation**, then add audited destination-side compatibility/port/storage diagnostics. No apply/cutover support.
4. With B3 isolated regressions verified, pursue the distinct G1 multi-service Linux acceptance matrix and F1–F4 migration/cutover/rollback implementation on disposable hosts. Do not count successful single-container smoke as multi-service or real-host readiness.

## Non-negotiable safety and completion gates

- `main` is the tested integration line; develop on task branches and merge reviewed PRs after applicable checks pass.
- The default Docker context and `/var/run/docker.sock` remain untouched; Caskel uses its own socket/profile.
- Do not attach production database state read-write to two live engines. Do not delete older profiles, Nix generations or backup copies as cleanup.
- CI uses disposable Linux runners. An optional GTK window is not claimed tested until built and exercised as a Linux GUI.
- 100% requires all scored deliverables and all stated acceptance gates, not merely a green compile or empty PR list.

**References:** [Linux-only audit](linux-only-audit.md), [Linux Docker migration plan](linux-migration-plan.md), [upstream policy](../UPSTREAM.md), [Nix/systemd setup](../nix/systemd/README.md). Update this ledger on substantive verified milestones; do not append chat transcripts or routine progress noise.
