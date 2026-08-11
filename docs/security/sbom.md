# Software Bill of Materials (SBOM)

Aether OS publishes dependency transparency for release artifacts. Full automated SBOM
generation is planned; this document describes the current stub and integration path.

## Current status (M2)

| Item | Status |
|------|--------|
| Pinned toolchain (`rust-toolchain.toml`) | Implemented |
| Committed `Cargo.lock` | Implemented |
| Release checksums (`SHA256SUMS.txt`) | Implemented in release workflow |
| SPDX / CycloneDX SBOM artifact | **Stub** — manual export below |

## Manual SBOM export (today)

From the repository root after installing [cargo-sbom](https://github.com/rust-embedded/cargo-sbom):

```bash
cargo install cargo-sbom
cargo sbom --output-format spdx_json_2_3 > dist/sbom.spdx.json
```

For the UEFI boot loader only:

```bash
cargo sbom -p aether-boot --target x86_64-unknown-uefi --output-format spdx_json_2_3 \
  > dist/sbom-aether-boot.spdx.json
```

## Planned CI integration

When SBOM generation is promoted from stub to required:

1. Add `cargo-sbom` install step to `.github/workflows/release.yml`.
2. Attach `sbom.spdx.json` (and optionally CycloneDX) to GitHub Releases alongside
   `aether-os-*.zip` and `SHA256SUMS.txt`.
3. Record SBOM hash in `SHA256SUMS.txt` for verification.

## Related

- [ADR-0007: Reproducible Builds Intent](../adr/ADR-0007-reproducible-builds-intent.md)
- [Threat model](./threat-model.md)
