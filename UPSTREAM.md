# Upstream tracking policy

Caskel is a Linux-first fork of
[`libracoder/kiri`](https://github.com/libracoder/kiri).
Our `main` is the stable, tested product branch. Upstream is a **reference**
for fixes, performance work, security patches and selected new features:
we never automatically merge its `main` into ours.

## Local Git remotes

Once the GitHub repository has been renamed to `sguzman/caskel`:

```fish
git remote set-url origin https://github.com/sguzman/caskel.git
```

Check before adding upstream:

```fish
git remote -v
```

If absent:

```fish
git remote add upstream https://github.com/libracoder/kiri.git
```

Fetch its history without merging:

```fish
git fetch upstream --prune
```

Inspect potentially new patches:

```fish
git log --cherry-pick --right-only --oneline main...upstream/main
```

## Branches

- `main`: tested Caskel release line.
- `feat/*`: Linux-first changes.
- `fix/*`: targeted repairs, including security fixes.
- `sync/*`: reviewable batches of upstream commits.

Create a `sync/*` branch from the current Caskel `main`, inspect
dependencies, cherry-pick or manually port appropriate upstream changes,
and run Linux Nix/CI and targeted runtime tests. Integrate through PRs
rather than rewriting `main`. A selective cherry-pick is not necessarily
standalone; dependent commits and conflict resolution may be required.

Review upstream security corrections promptly even when other changes
are deferred. Track accepted upstream commit IDs in sync PRs.

## Deliberately retained upstream internals

For manageable future merges, the Cargo crate/package names `kiri`,
`kiri-init`, internal symbols and existing `KIRI_*` configuration
variables remain for now. Public Linux interfaces use `caskel`:
the executable, Nix package/profile, systemd service, Unix socket, default
local state/context and data store. This is a conscious staging decision.

Keep original Apache-2.0 and NOTICE attribution. Do not reuse upstream's
protected name/logo as the project's own brand.

## Deployments

Build a pinned Git revision, test it, then promote its Nix output into the
dedicated root-owned Caskel profile. Restart services deliberately; do not
auto-deploy an upstream `main` or erase prior Nix generations or runtime data
without verified rollback.
