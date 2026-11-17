# Caskel identity and compatibility ledger

Caskel's **public product name, CLI, standalone SVG artwork, release
documentation and Linux UI** are Caskel, not upstream Kiri. Historical
copyright/legal attribution continues to identify Kiri accurately.

## Public identity

- Public CLI/daemon: `caskel`, with `/version` reporting Caskel as the
  Platform/Component and `CaskelVersion`.
- Linux agent: `caskel-init` (with a transitional `kiri-init` executable
  compatibility link for old launch configurations).
- Nix profile/service/socket/data:
  `/nix/var/nix/profiles/caskel`, `caskel.service`,
  `/run/caskel/docker.sock`, `/var/lib/caskel-daemon/data`.
- Local Linux share account configuration: `~/.caskel/share.json`, not
  a silent reuse of historical Kiri Cloud credentials.
- Local diagnostic text: `Caskel diagnostics report`, default
  `caskel-report-<timestamp>.txt`; existing historical reports remain
  readable without rename or deletion.
- Upstream Kiri Cloud account URLs remain labeled **Kiri Cloud**, not
  Caskel Cloud. No Caskel Cloud service is offered. Connections to the
  inherited Kiri-operated domain require an explicit
  `CASKEL_ALLOW_LEGACY_KIRI_CLOUD=1` opt-in.

## Transitional protocol compatibility (not branding)

The following internal interfaces must not be changed casually:

| Surface | Why it remains | Exit condition |
| --- | --- | --- |
| `KIRI_SOCKET`, `KIRI_DATA_DIR`, `KIRI_AGENT`, `KIRI_SOCKET_GROUP`, port variables | Existing agent/service overrides and tests | Canonical `CASKEL_*` environment options now win, original options remain as compatible fallbacks; deployed unit migration requires separate opt-in |
| `/kiri/*` Engine extension endpoints | Internal GUI/daemon API and old clients | Versioned alias+client update with regression tests |
| `KiriVersion` in Engine API | Legacy clients and daemon version checks | Preserve alongside `CaskelVersion` until callers migrated |
| `/run/kiri`, `~/.kiri` legacy paths | Earlier Kiri rollback and some agent internals | Verify producers, old profile and storage migration before retiring |
| `kiri.local`, `kiri.localhost` domains | DNS/TLS name constraints and persisted certificate authority | Design safe dual-name support, certificate migration and DNS rollback |
| `dev.kiri.*` labels and internal K8s naming | Existing image/network/extension identity | Backward-compatible alias migration and state-owner audit |
| `kiri-init` executable alias | Agent launch expectations in old paths | Validate deployed binary selection with rollback |

The remaining terms above are **not** a reason to show old Kiri logos or
claim Kiri's release channel, security policy, support address or branded
diagnostics as Caskel services.

## Scope and acceptance

- Branding commits require Nix/Rust tests, UI/i18n checks, CodeQL and Speed.
- The pinned headless Nix build is supported. PR #32 adds a Linux
  GTK/WebKitGTK 4.1 compile gate to CI for the optional window; this is
  distinct from **real desktop GUI acceptance**, which remains pending.
- The optional Linux GTK About panel identifies its package as Nix-managed;
  it must not check or advertise inherited Kiri release feeds, upstream
  `.deb`/`.rpm` downloads, or automatic in-app installation as Caskel updates.
  UI tests cover the pinned Nix state and external Kiri Cloud disclosures.
- The Caskel Linux diagnostics screen must name the upstream-operated Kiri Cloud
  service as the actual recipient, explain the explicit
  `CASKEL_ALLOW_LEGACY_KIRI_CLOUD=1` opt-in before uploading, and not imply
  Caskel operates the service. New Linux-only disclosures use the existing
  translation catalog keys in all 37 supported locale files. GTK/window
  desktop acceptance is still distinct from headless Chrome and compile CI.
- Do not auto-switch Docker's default context, rewrite old Kiri files, or
  restart either old Kiri or production Docker.
- Do not bulk-replace protocol/setting identifiers without versioned
  compatibility tests and an explicit rollback.

## Repository profile settings (owner action completed)

Source-level branding is independent of GitHub's repository profile.
The repository owner corrected the GitHub **About** metadata on 2026-11-17,
confirmed from the repository API:

- The description is now **"Linux-first Rust container engine with
  Docker-compatible API, pinned Nix builds and isolated systemd integration."**
- The inherited homepage `https://runkiri.com` has been **cleared**.
- Topics remain optional (currently empty). Suggested truthful topics include
  `rust`, `linux`, `containers`, `nix`, `docker-compatibility`.
- GitHub Issues are currently disabled, so issue templates remain dormant
  unless the owner enables them separately.

The connected GitHub source/PR interface cannot mutate repository-profile
metadata; that owner-only action is **complete** and is not an outstanding
branding blocker. Historical attribution and GPL source obligations remain
unchanged. **Real Linux GTK desktop acceptance is still outstanding.**
