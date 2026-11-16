# Nix packaging for the Kiri fork

This directory builds the engine and agent for **x86_64 Linux** directly from
the checked-out Rust workspace. It does **not** call the upstream Docker-based
`cargo xtask linux`, copy files into `/usr/local`, change your Docker socket,
start a daemon, or write any privileged system configuration.

## Local build on EndeavourOS (fish)

After installing Nix and enabling the `nix-command` and `flakes`
experimental features:

```fish
git clone https://github.com/sguzman/kiri.git
```

```fish
cd kiri
```

The packaging changes live on `feat/nix-flake` until the CI build passes
and the change is merged:

```fish
git switch feat/nix-flake
```

```fish
nix build .#kiri
```

```fish
ls -lh result/bin/kiri result/bin/kiri-init
```

The `result` symlink points to an immutable Nix store output. Removing it
does not remove Docker, Kiri container data, or system services (none are
created by this build).

The initial `flake.lock` is generated and committed automatically by
the feature-branch CI *after* a successful build. If you check out the
branch before CI produces that commit, the first local Nix command will
resolve the input and generate an untracked `flake.lock` until it is
committed upstream. For a fully pinned build, pull the committed lock.

## Development

```fish
nix develop
```

```fish
nix flake check
```

`nix flake check` currently validates that both production executables
compile. It does not run privileged container/integration tests. The
optional Linux GUI (GTK/WebKit) is also **not** included in this minimal
package.

## Optional Arch/EndeavourOS systemd integration

The Nix package includes a **disabled, opt-in** service template. See
[`nix/systemd/README.md`](systemd/README.md) for a dedicated root-owned Nix
profile, explicit `systemctl link`, safe startup, and complete reversible
removal instructions. It uses the pre-existing `docker` group rather than
creating a new privileged `kiri` group. Docker's socket remains untouched.

**Do not start Kiri before reviewing its networking and host access.**
Container state cleanup is separate from removing the package.

## Fork branching rules

- `main`: stable, intended for daily use. No direct experimental commits.
- `feat/*`: one feature at a time; merge with a PR after checks.
- `fix/*`: bugfixes; PR into `main`.
- `chore/upstream-*`: merge/sync upstream changes and resolve conflicts;
  PR into `main` after testing. Do not force-push `main` to upstream.
- No permanent `develop` branch. Avoid accumulating divergent branches.
- Prefer squash merges for fork-specific work and document any upstream
  divergence. Keep upstream engine sources unchanged when only packaging
  changes are needed.

The source code is Apache-2.0. The upstream Kiri name and logo are not
covered by that license; a separately branded distributed product needs
its own name and identity. This fork currently only adds packaging.

## Updates and reversibility

Update `Cargo.lock` only when changing Rust dependencies; update the
Nixpkgs input via `nix flake update nixpkgs` in a dedicated PR. CI must
pass before the new lockfile enters `main`.

No `doas`, `sudo`, systemd changes, Docker configuration edits, or
new system groups are required to build or enter the development shell.
