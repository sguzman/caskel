# Caskel systemd integration on Arch / EndeavourOS

Caskel and its Nix profile are **separate** from the original Kiri test
installation and from production Docker. No action below alters Docker's
default context, `/run/docker.sock` or any existing Docker container.

## Layout

| Resource | Existing Kiri test | New Caskel |
|---|---|---|
| Profile | `/nix/var/nix/profiles/kiri` | `/nix/var/nix/profiles/caskel` |
| Service | `kiri.service` | `caskel.service` |
| Socket | `/run/kiri/docker.sock` | `/run/caskel/docker.sock` |
| Persistent state | Under `/var/lib/kiri-daemon` | `/var/lib/caskel-daemon` |
| Image store | Existing Kiri inventory | `/var/lib/caskel-daemon/data` |

**No data is migrated automatically.** The old Alpine image and old Kiri
runtime state remain available for rollback. The root-owned Caskel service
has access to the shared Linux host, including bridges, nftables, mounts and
cgroups, and runs with effectively root-equivalent privileges.

Do not run the old Kiri and new Caskel daemons at the same time: their
network management has shared assumptions. The new unit also declines
to start when `/run/kiri/docker.sock` exists. Docker can continue running.

## Build and inspect (unprivileged)

```fish
nix build .#caskel
```

```fish
ls -l result/bin/caskel result/bin/caskel-init result/bin/kiri-init result/lib/systemd/system/caskel.service
```

```fish
cat result/lib/systemd/system/caskel.service
```

The flake builds the public `caskel` CLI and canonical `caskel-init`
Linux agent. The `kiri-init` executable is retained only as a compatibility
symlink for legacy launch configurations. The flake packages the systemd
unit but does **not** install or start it.

## Install into a distinct Nix profile

```fish
doas nix profile install --profile /nix/var/nix/profiles/caskel (realpath result)
```

```fish
doas systemctl link /nix/var/nix/profiles/caskel/lib/systemd/system/caskel.service
```

```fish
systemctl cat caskel.service
```

```fish
fish_add_path -U /nix/var/nix/profiles/caskel/bin
```

The service is now registered but **not started or enabled at boot**. Do
not `realpath` the unit when linking; its location under the Nix profile
should remain stable across profile generations.

## Controlled smoke test

When ready to pause the **experimental** Kiri daemon, but not Docker:

```fish
doas systemctl stop kiri.service
```

```fish
systemctl is-active kiri.service
```

Expect `inactive`. Then:

```fish
doas systemctl start caskel.service
```

```fish
systemctl status caskel.service -l --no-pager
```

```fish
caskel version
```

```fish
caskel run --rm --network none alpine:3.21 echo CASKEL_OK
```

The first test might pull the Alpine image again: Caskel deliberately
has a new inventory. Follow with `docker ps` to confirm production
Docker services are still healthy.

Optional: create a named Docker context without changing the default,
after inspecting any existing one with `docker context inspect caskel`:

```fish
docker context create caskel --docker host=unix:///run/caskel/docker.sock
```

Do **not** use `docker context use caskel` on this production host.

## Rollback

Stop Caskel before restarting the old experimental Kiri service:

```fish
doas systemctl stop caskel.service
```

```fish
doas systemctl start kiri.service
```

To fully unregister the new system service, use
`doas systemctl disable --now caskel.service`; once stopped, remove the
separate package with
`doas nix profile remove --profile /nix/var/nix/profiles/caskel --all`.
Do not remove `/var/lib/caskel-daemon` or old Kiri data without a separate
backup and explicit approval. The working Docker runtime is outside all
these profile operations.

For upgrades, replace only the dedicated Caskel profile after review and
restart the Caskel service explicitly. For rollback, use
`doas nix profile rollback --profile /nix/var/nix/profiles/caskel`
before deliberately restarting its service.
