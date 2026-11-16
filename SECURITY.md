# Caskel security policy

Caskel is a Linux host container engine. Its agent can exercise Linux
namespaces, cgroups, mounts, networking and seccomp with root-level
authority. Access to its Docker-compatible Unix socket is effectively
root-equivalent. Treat reports of isolation bypass, access control,
container escape, data deletion or host compromise as security-sensitive.

## Reporting

Do not publish exploit details or credentials in a public issue.
Use GitHub's **private vulnerability reporting** for this repository if
available (Security → Report a vulnerability). If that option is unavailable,
contact the maintainer through an established private channel; do not
post a proof of concept containing secrets publicly.

Provide the Caskel commit/version, Linux distribution and kernel,
reproduction steps in a disposable environment, impact and relevant
redacted logs. Do not upload live Docker socket credentials or private
volume contents.

There is **no claimed guaranteed response SLA**, release feed, automatic
updater or blanket support for historical upstream Kiri versions.

## Scope and fixes

- Linux host/runtime isolation; seccomp, capabilities, namespaces, mounts
  and cgroups.
- The daemon/agent control sockets and Docker-compatible HTTP API.
- Published ports, firewall/nftables rules, DNS, local names and TLS.
- Image supply chain and archive extraction.
- Persistent data safety, permissions, runtime restart/recovery and
  Docker-to-Caskel migration/rollback.

Review upstream security fixes promptly but integrate them selectively
after Linux-specific tests. An isolated green Nix build is not a substitute
for privileged container and networking acceptance. Never test an exploit
against a production host.

See [upstream policy](UPSTREAM.md), [release process](RELEASES.md),
and [verified roadmap](docs/roadmap.md).
