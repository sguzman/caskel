# Packaging Caskel on Linux

Caskel's supported packaging route is the reproducible **Nix flake** and
opt-in, isolated `caskel.service`. The package and service do not install,
start or replace a Docker daemon or Docker's default socket.

```fish
nix flake check
```

```fish
nix build .#caskel
```

See [Nix packaging](../nix/README.md) and the
[reversible Arch/EndeavourOS service guide](../nix/systemd/README.md).
Building the Nix derivation does not install or activate a service.

The original Kiri `.deb`, `.rpm`, and `contrib/linux` packaging trees
were **retired** during Caskel's Linux-first transition. They installed an
independent `kiri.service`, used old socket/data paths, could manipulate
`/run/docker.sock`, and relied on release scripts no longer present in
this fork. They must not be repurposed as Caskel packaging.

Any future Debian/RPM distribution must be authored and acceptance-tested
specifically for Caskel's isolated service, explicit opt-in install, and
Docker-coexistence policy. Do not silently reintroduce socket symlinks,
package upgrade restarts, group changes or data purge hooks.

Historical legal notices and upstream Apache-2.0 attribution remain in
[LICENSE](../LICENSE) and [NOTICE](../NOTICE). The GPL guest-kernel
corresponding-source materials remain separate pending provenance review.
