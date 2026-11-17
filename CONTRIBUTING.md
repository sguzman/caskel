# Contributing to Caskel

Caskel is an independently maintained **Linux-first** Rust container engine.
Contributions should preserve Docker API compatibility where intentional,
minimize privileges, respect upstream provenance and keep deployed user
data safe.

## Development scope

- Target tested Linux behavior, starting with `x86_64-linux`.
- Keep changes focused and reviewable. Develop on `feat/*`, `fix/*`,
  `docs/*` or attributed `sync/*` branches; do not fast-forward Caskel
  to upstream Kiri's main branch.
- Preserve production Docker's default socket, context and state. Caskel
  uses isolated paths and an explicitly installed service.
- Never silently repurpose an old Kiri service, package hook, install script,
  updater, or protected logo as Caskel.
- Treat `KIRI_*` variables, private agent protocols and persisted state as
  compatibility-sensitive. Map and test consumers before renaming them.

## Build and checks

Use the pinned Nix environment:

```fish
nix develop
```

```fish
nix flake check
```

```fish
nix build .#caskel
```

```fish
nix develop --command cargo fmt --all -- --check
```

```fish
nix develop --command cargo test --locked -p caskel -p caskel-proto -p caskel-agent-transport
```

GitHub Actions runs additional privileged Linux agent and container smoke
tests on **disposable runners**, strict UI/i18n checks, CodeQL and
performance comparisons. Never reproduce privileged tests or benchmark
cleanup operations against a host with important Docker workloads.

## Safety review for every change

A pull request should explain the source-level motivation, files and
interfaces touched, tests actually run, known gaps, rollback implications,
and whether root privileges, network rules, mounts, storage or the daemon
socket could be affected.

Migration work must first inventory Docker state read-only, then show a
deterministic dry-run, and only later implement explicit per-service
cutover and rollback. Do not attach a live database to two engines at
once or make a Docker context switch automatically.

For selected upstream fixes, attribute the upstream commit and
dependencies in the PR. Keep original `LICENSE`/`NOTICE` and required
copyright notices intact. Follow applicable contribution/license
requirements already recorded in the repository; do not erase author
credits in cleanup patches.

See [development guide](docs/development.md),
[security policy](SECURITY.md),
[upstream tracking](UPSTREAM.md) and
[verification roadmap](docs/roadmap.md).
