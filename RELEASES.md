# Caskel releases and reproducibility

Caskel is an independent Linux-first fork of
[libracoder/kiri](https://github.com/libracoder/kiri). It does not use
upstream Kiri's download feeds, Mac notarization, Windows installers,
private signing infrastructure, or release promises.

## Current distribution

The supported artifact today is a **pinned Nix flake for x86_64-linux**.
It packages the headless Rust engine, Linux agent and an optional systemd
unit. The flake lock pins the Nix dependency graph; Cargo.lock pins Rust
dependencies. CI evaluates and builds the package and tests Linux runtime
behavior on a disposable host.

This source repository does **not yet publish a separately verified
Caskel binary release channel**. Do not infer a new stable tag, signing
key, auto-update feed, distribution repository, or guaranteed support
window from upstream's old release notes.

From a pinned, reviewed checkout:

```fish
nix flake check
```

```fish
nix build .#caskel
```

These commands do **not** install or start a privileged service.

## Promotion and rollback policy

1. Review exact source commit and `flake.lock`/`Cargo.lock` changes.
2. Require Nix build, Rust/unit, privileged disposable runtime smoke,
   UI/i18n, CodeQL and performance acceptance where applicable.
3. Inspect the generated unit and isolated socket/data paths.
4. Test disposable workloads and preserve a known-good Nix profile generation.
5. Deliberately promote the pinned derivation into the separate root-owned
   Caskel profile. Do not automatically restart a running service.
6. Verify production Docker was unaffected; retain rollback assets.
7. For any future service migration, verify application-consistent backups
   and restoration before switching production traffic.

The service integration guide provides explicit opt-in install and
rollback commands. Package creation and profile promotion are separate
operations.

## Upstream and attribution

Accept upstream changes selectively through reviewed `sync/*` branches.
Keep Apache-2.0 `LICENSE`/`NOTICE` and original copyright provenance.
Historical GPL guest-kernel corresponding-source materials are retained
pending a separate provenance/legal review. The
[historical release provenance ledger](docs/kernel-provenance.md) distinguishes
upstream source-asset metadata from unverified archive contents and inherited
Caskel Git tags.

See [upstream policy](UPSTREAM.md),
[Nix packaging](nix/README.md),
[systemd installation/rollback](nix/systemd/README.md),
and [verified development roadmap](docs/roadmap.md).
