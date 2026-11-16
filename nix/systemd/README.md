# Optional systemd integration on EndeavourOS / Arch

This is an opt-in bridge between a **root-owned Nix profile** and Arch's
system systemd manager. It is not a NixOS module. The Nix derivation merely
ships the unit file: `nix build` never installs or starts anything.

The unit is for native x86-64 Linux only and intentionally avoids global
Docker socket changes, CLI replacement, or automatic service startup.

## Requirements and safety boundaries

- Nix with `nix-command` and `flakes`, systemd, and a working Docker CLI.
- `doas` for the few explicit root operations.
- The existing `docker` group, used for socket access. **Docker group
  membership already grants effectively root-equivalent access.** The
  `kiri` service uses the same group; it never modifies group membership.
- The `/nix/var/nix/profiles/kiri` profile is **dedicated to Kiri only**.
- Kiri listens on `/run/kiri/docker.sock`, not `/run/docker.sock`.
- The optional local-domain HTTP/HTTPS routers use loopback ports 18080 and
  18443 instead of 80 and 443. Kiri also listens for its optional DNS on
  `127.0.0.1:53535`, **not** the system's port 53.
- When started, the privileged engine can still change bridges, mounts,
  cgroups and nftables rules. Inspect the system before starting it next to
  production containers. The unit does not attempt to sandbox those features.
- Containers and engine state are separate from the Nix profile and persist
  under `/var/lib/kiri-daemon`. Removal does **not** delete them.
- Do not activate or enable the service automatically on first installation.

## Build and inspect (unprivileged)

From the repository's `main` branch, after the systemd integration PR
is merged:

```fish
nix build .#kiri
```

```fish
ls -l result/bin/kiri result/bin/kiri-init result/lib/systemd/system/kiri.service
```

```fish
cat result/lib/systemd/system/kiri.service
```

## Install into a dedicated root-owned Nix profile

Before installing, verify there is no pre-existing Kiri service or profile
from another experiment: `systemctl cat kiri.service` and
`ls -ld /nix/var/nix/profiles/kiri` (an absent path is expected).

```fish
getent group docker
```

```fish
doas nix profile install --profile /nix/var/nix/profiles/kiri (realpath result)
```

```fish
doas systemctl link /nix/var/nix/profiles/kiri/lib/systemd/system/kiri.service
```

```fish
systemctl cat kiri.service
```

Use the stable profile path above rather than `realpath`: linking a specific
`/nix/store` derivation would leave systemd pointing at an old unit after
Kiri is upgraded or rolled back. `systemctl link` registers the unit without
starting or enabling it.

At this point systemd knows about Kiri, but it is **not started or enabled**.

## Opt-in smoke test, after host network review

```fish
doas systemctl start kiri.service
```

```fish
systemctl status kiri.service --no-pager
```

```fish
docker context create kiri --docker host=unix:///run/kiri/docker.sock
```

```fish
docker --context kiri info
```

```fish
docker --context kiri run --rm --network none alpine:3.21 true
```

If the `kiri` context already exists, inspect it with
`docker context inspect kiri` instead of recreating it. Always pass
`--context kiri` explicitly: never use `docker context use kiri` during
the coexistence test. On failure, stop Kiri and inspect
`journalctl -u kiri.service -b --no-pager`.

## Stop, undo, and keep your data

```fish
doas systemctl disable --now kiri.service
```

```fish
doas systemctl daemon-reload
```

```fish
doas nix profile remove --profile /nix/var/nix/profiles/kiri --all
```

```fish
docker context rm kiri
```

`systemctl disable` removes links created by `systemctl link`, even if
the service was never enabled. The dedicated Nix profile's previous
generations remain available for rollback; use
`doas nix profile wipe-history --profile /nix/var/nix/profiles/kiri`
**only when you no longer want those rollback generations**.

No commands above delete `/var/lib/kiri-daemon`, Docker data, images,
networks, or your existing system services. Do not erase Kiri runtime data
until you have inspected or backed it up separately.

## Upgrade/rollback model

The unit executes the stable root profile path, not a particular store hash.
Future updates should replace the Kiri item in the dedicated profile and
restart `kiri.service` explicitly, then verify the engine. Do not update
or garbage-collect the Nix store while an experimental Kiri daemon is still
running without validating the package transition.

The `nix profile rollback --profile /nix/var/nix/profiles/kiri` operation
can select a previous package generation. It still requires a deliberate
service restart to take effect.
