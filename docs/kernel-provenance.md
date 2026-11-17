# Historical guest-kernel release provenance (C4)

**Evidence status, 2026-11-17:** upstream GitHub release-asset **metadata**
reviewed; archive contents, historical binary correspondence, and GPL
compliance **not independently verified**. This is a technical preservation
ledger, not a legal opinion or a new Caskel source offer.

## Distribution boundaries

- **Caskel:** the fork has inherited `v0.1.*`/`v0.2.*` git tags but
  **no published GitHub Releases** at the time of the audit. The inherited
  `docs/releases/` notes are upstream history, not certified Caskel
  release claims. Caskel's native Nix package builds Rust engine + agent
  without `guest/` build inputs (verified by PR #44 and its Nix CI).
- **Upstream Kiri:** the historical `guest/kernel/SOURCE_OFFER.md` names
  `download.runkiri.com` and states a three-year term from each Kiri
  release. Preserve that original text for provenance; do not rebrand it
  into a Caskel promise or assume the upstream domain still serves those
  files. `scripts/ci/notices.sh` retains the offer in generated NOTICE.

## Published upstream GitHub release metadata

The upstream GitHub Releases API currently lists six releases with
`Kiri-VERSION-kernel-source.tar.xz` source assets. Dates, sizes and SHA-256
digests below are **GitHub-provided asset metadata**, not locally
calculated hashes from downloaded archives.

| Upstream release | Publication date (UTC) | Archive size (bytes) | GitHub-reported SHA-256 |
| --- | --- | ---: | --- |
| [v0.1.14](https://github.com/libracoder/kiri/releases/tag/v0.1.14) | 2026-11-02 | 160865588 | `5d5b5cdaa9538fb95158fee4a5c73c8d1db6beb783b0155dc43d70826e6e104b` |
| [v0.1.15](https://github.com/libracoder/kiri/releases/tag/v0.1.15) | 2026-11-03 | 160856488 | `2ad9aa4b56688ce2db08eb5719d97e45303824022a48a4b9e08d0f2739951985` |
| [v0.1.17](https://github.com/libracoder/kiri/releases/tag/v0.1.17) | 2026-11-03 | 160856888 | `e4339144221287f18cd11cb3ce8da05f11525f31d6fc7b6edb68b23434b7459f` |
| [v0.1.18](https://github.com/libracoder/kiri/releases/tag/v0.1.18) | 2026-11-06 | 160859560 | `900df0a55f5df8d1b0ec4865ba0265ebc76037c29b3060beee8f5026777dc600` |
| [v0.2.0](https://github.com/libracoder/kiri/releases/tag/v0.2.0) | 2026-11-10 | 160881132 | `526cdfc6b88d207b25c93225e6d5cc7051c85381757e9eafcf465ccfc5c000d2` |
| [v0.2.1](https://github.com/libracoder/kiri/releases/tag/v0.2.1) | 2026-11-15 | 160882168 | `703a9a9fa8e90d3435c94c1163fab54631a58c2ff50a24a18ec3399272b6c819` |

This metadata establishes that source archives were **listed** with
upstream release assets. It does **not** establish live download
availability, archive completeness, exact correspondence to every
binary distributed, or fulfillment of a particular legal obligation.
Other older upstream tags or inherited release-note files are not
evidence of GitHub binary publication.

## Retained source materials

- `guest/kernel/SOURCE_OFFER.md`: original offer, left unchanged.
- `guest/kernel/build.sh`: historical Apple guest-kernel builder. The
  current checkout's default `KVER=6.18.58` does **not** prove the
  same version was used for every earlier release.
- `guest/kernel/kiri.config`, `guest/kernel/kiri-x86_64.config`,
  and five `guest/kernel/patches/*.patch` files.
- `scripts/ci/kernel-source.sh`: historical source archive generator
  that fetches a kernel tree and applies checkout patches. It has
  **not** been executed or compared to published archives in this audit.
- `scripts/ci/notices.sh` and `NOTICE`: historical license,
  attribution and offer retention.

## Verification still required before legal conclusions

For each actual upstream binary release in scope, a qualified reviewer
must establish: (1) which kernel object was distributed and when;
(2) the exact corresponding-source archive and immutable checksums;
(3) archive readability and complete source/configuration/patch/build
instructions; (4) binary-to-source build correspondence; (5) applicable
notices, distribution channels, duration and downstream obligations.
Git tags and GitHub asset metadata alone cannot establish all of this.

**Disposition:** retain historical guest-kernel materials and the source
offer unchanged. Do not promise a Caskel kernel-source channel or remove
the offer without release-specific evidence. C4's remaining provenance
review is open.
