# Linux-only audit: Caskel migration inventory

**Status:** historical source inventory from 2026-11-16, followed by
verified disposition notes. Entries under "Initial dependency observations"
describe **before-cleanup paths**, not necessarily files still present.
This is **not approval to delete all listed files**. Update it as each
removal is verified. Caskel is a Linux-host container runtime with a Docker-compatible API, not a macOS/Windows desktop distribution.

## Ground rules

- **Protect the working Linux baseline.** Keep a reproducible Nix build and a stable `main`; make focused changes through PRs.
- **Preserve production Docker.** Never repoint its default socket, remove Docker data or migrate live databases as part of repository cleanup.
- **Separate public identity from historical provenance.** Replace upstream logos, product identifiers and public names. Retain the Apache-2.0 `LICENSE`, `NOTICE`, required copyright headers, upstream commit references and unambiguous attribution.
- **Don't delete by filename alone.** A Linux-relevant runtime file may still be named after upstream or describe a VM.
- **Keep security ports reviewable.** Confirm Linux unit/integration behavior before upgrading a host profile. Prefer selective, attributed upstream fixes.

## Initial dependency observations

| Location | Linux relevance | Proposed treatment |
|---|---|---|
| `crates/kiri/src/daemon/`, `crates/kiri-init/src/linux/`, `crates/kiri-proto/`, `crates/kiri-image/` | Core engine, agent, protocol, image store | **Keep**; audit public naming, protocol compatibility and safety |
| `crates/kiri-vmm/src/process.rs` | Linux host agent launcher and Unix-socket transport; used by `crates/kiri/src/guest.rs` | **Keep**. Refactor the misleading VM facade only after callers and integration tests are mapped |
| `crates/kiri-vmm/src/apple.rs` | Apple Virtualization.framework implementation | Remove after disentangling `kiri-vmm` interfaces and verifying Linux build |
| `crates/kiri/src/guest.rs` | Contains **both** Linux and macOS paths | Split or simplify; **do not remove** the Linux agent connection |
| `crates/kiri-win/`, `installer/windows/`, `contrib/windows/`, `contrib/wsl/` | Windows/WSL product and packaging | Candidate removal in a focused platform-pruning PR |
| `crates/kiri/src/wsl.rs`, `tests/wsl.sh`, `scripts/windows-engine.sh` | Windows/WSL compatibility | Candidate removal; check references and CI first |
| `scripts/dmg-background.swift`, `scripts/dmg-settings.py`, `tests/finder-mount.sh` | macOS DMG/Finder integration | Candidate removal after confirming no Linux build dependency |
| `guest/kernel/` | Bundled guest VM kernel | Inspect whether any Linux-host build, test or release still needs it; **do not delete yet** |
| `packaging/linux/`, `contrib/linux/` | Legacy Linux distribution integration | Audit for outdated `kiri` commands/services and either adapt or replace with the Nix/systemd path |
| `nix/`, `flake.nix`, `flake.lock` | Tested Linux packaging | Keep, strengthen tests and rollback instructions |
| `crates/kiri/src/ui/` | Linux GTK/WebKit UI is optional; headless Nix package does not include it | Explicitly decide Linux GUI scope; do not conflate with macOS-only UI |
| `brand/logo/`, `brand/icon/`, `docs/images/kiri-window.png` | Original product assets | Remove from Caskel-facing surfaces and replace with **new** assets if needed; preserve legal provenance in text |
| `README.md`, `docs/development.md`, `docs/features.md`, `RELEASES.md`, `CONTRIBUTING.md`, `SECURITY.md` | Public and contributor documentation | Rewrite to describe tested Linux behavior; retire unsupported desktop-platform promises |

## Public naming / state compatibility audit

Current Cargo package/crate names and the public agent binary use **Caskel**.
Older `KIRI_*` settings (as compatibility fallbacks), `/kiri/*` HTTP
extension paths, historic `~/.kiri` paths, and `*.kiri.localhost` DNS/TLS
identities remain where they are external or persisted interfaces. These
must not be mass-renamed without migration and rollback tests. Current CLI
help no longer promotes macOS/WSL product targets.

Before changing these, map every producer/consumer, persisted state field, socket, Docker context, environment variable, test and service unit. Plan an explicit compatibility/migration strategy for each external interface, then remove aliases only when verified. The running Caskel install uses the dedicated `/nix/var/nix/profiles/caskel` profile, `/run/caskel/docker.sock` and `/var/lib/caskel-daemon/data`. Old Kiri's profile/data are rollback assets, not cleanup targets.

## Branches and upstream

- `main`: independently tested Linux release line.
- `feat/*`: targeted features.
- `fix/*`: targeted bug/security fixes.
- `sync/*`: selectively ported upstream changes with source commit IDs, prerequisite analysis and Linux tests.
- `docs/*`: documentation and audit branches.

Never click **Sync fork**, hard reset Caskel `main` to upstream, or auto-deploy upstream commits. Review and deliberately cherry-pick/reimplement source changes. See [UPSTREAM.md](../UPSTREAM.md).

## Verification gates before and after platform pruning

1. Nix `x86_64-linux` flake/package builds `caskel`, `caskel-init`
   (and the transitional `kiri-init` alias) plus the isolated systemd unit.
2. Native Linux Rust unit tests for agent, protocol, security-policy compiler and core daemon.
3. Disposable containers: pull/run/exec/logs/stop/remove, persistence through a daemon restart, and cleanup after unexpected process exit.
4. Networking: isolated bridge, DNS, published-port collisions with existing Docker, firewall rules and Compose behavior.
5. Volumes/binds: write, ownership, read-only and restart behavior. Never point two active engines at the same database storage.
6. Validate production Docker remains healthy; keep the previous Nix generations and old Kiri profile for rollback.
7. Confirm README/help/docs and release jobs no longer offer unsupported macOS/Windows targets.

A green Nix compilation is **not** a substitute for privileged Linux integration testing.

## Work sequence (reviewable PRs)

1. Finish current reliability/security ports and establish the runtime acceptance baseline.
2. Remove clearly isolated Windows installer/WSL assets and dead CI jobs; run all checks.
3. Decouple Apple VM types from the Linux agent transport; only then remove Apple VM code and VM-only kernel artifacts if unused.
4. Decide Linux GTK UI support and simplify cross-platform CLI dispatch without deleting shared Linux features.
5. Rebrand agent, Cargo crates, public contexts, local-domain handling and configuration variables with explicit upgrade/rollback strategy.
6. Replace upstream visual assets; rewrite project philosophy, development, security, contributor and release docs to match proven behavior.
7. Develop native Docker migration via **read-only inventory → dry run → reversible per-service cutover**. See [linux-migration-plan.md](linux-migration-plan.md).

## Verified follow-up (November 2026)

- PR #12 removed unreachable Apple guest boot and disk helpers while preserving the Linux host agent transport and `KIRI_AGENT` compatibility.
- PR #13 removed the retired AppKit backend and 14 unused Mac-only translation keys in each of 37 locales. The optional Linux GTK window still needs its own build/QA milestone.
- The upstream `scripts/kiri-linux.sh` WSL rootfs assembler references the already removed `contrib/wsl` tree. The Mac `scripts/ci/guest-cache.sh`, `scripts/ci/linux-packages.sh` and upstream release-feed `scripts/ci/test-version.sh` target unsupported release paths; these were retired by the following focused cleanup.
- **Retain** `guest/kernel/` and `scripts/ci/kernel-source.sh` while auditing GPL-2.0 guest-kernel provenance and historical source-offer obligations. Removing obsolete compiled VM kernel support does **not** justify deleting notices or corresponding-source artifacts automatically.
- Legacy `packaging/linux/` and `contrib/linux/` are not the Nix-installed Caskel service. Their old `kiri.service` paths must never be installed as Caskel or used to displace a working `/run/docker.sock`; assess and retire or reimplement separately.

## Repository settings to address separately

At audit time, the GitHub repository description still says “macOS, Windows and Linux”; GitHub Issues are disabled; and `main` is not marked protected. These are repository administration settings, not runtime bugs. Enable Issues if desired for tracking Linux regressions and consider branch protection requiring Linux CI. The GitHub connector cannot apply all repository administration changes.

## Open questions (resolve from source/tests rather than assumptions)

- Which `kiri-vmm` interface fields can disappear without changing the Linux transport protocol?
- Does the optional Linux GTK window remain in Caskel's scope, and how should it be packaged?
- Which `guest/kernel` artifacts are still legally/technically required after removing VM releases?
- Which `KIRI_*` interfaces must be accepted temporarily to preserve existing systemd installations?
- Which automated tests truly exercise Linux privilege boundaries, data persistence and networking?

## Upstream brand assets (November 2026)

The old `brand/` directory contained Kiri-specific generated SVGs, PNGs,
Windows ICOs and an upstream logo generator. These were **not** Caskel
artwork. The optional Linux GTK window originally embedded the Kiri 512px
PNG, so removing the directory required first decoupling its icon loader.
The GTK window now requests the desktop's generic application icon and
advertises the `caskel` application identifier. This neutral placeholder
is not a final logo or a completed review of embedded HTML/SVG UI branding.
The original Apache-2.0 LICENSE/NOTICE and GPL kernel sources were preserved.

## Visible Linux UI translation identity

The 37 locale catalogs use stable English strings as lookup keys. The Linux
page and Rust UI now replace the user-visible product proper name **Kiri**
with **Caskel** at render time rather than renaming every key or discarding
the translations. The legacy source strings, including placeholders, remain
stable for upstream ports. Linux-specific tests cover the Spanish page and
English, Spanish and Japanese native translations.

Never pretend external Kiri Cloud or Kiri Share is a Caskel-operated service.
The brand substitution explicitly preserves those provider names, as well as
protocol variables `KIRI_*`, legacy paths (`~/.kiri`, `/run/kiri`) and
`*.kiri.local` names that are not migrated yet. These are compatibility
interfaces, not Caskel's public marketing name.

## Branding disposition (current)

- PRs #21, #22, #25–#29 renamed the public API, Linux UI, original SVG
  assets, project/community templates, Cargo packages, diagnostics and
  local share configuration; all passed four workflow suites.
- The old Kiri screenshot at `docs/images/kiri-window.png` and unused
  macOS DEV icon source at `xtask/dev-icon.swift` were removed from the
  current source. The Apple entitlement file was also retired.
- The actual product's license and original upstream copyright notices
  stay intact. Guest-kernel GPL-2.0 corresponding-source materials remain
  retained pending an explicit provenance/legal review.
- Current, intentionally retained compatibility identities are itemized in
  [the branding ledger](branding-compatibility.md).
- No actual running host service or Docker installation was changed during
  these repository-only branding patches.
