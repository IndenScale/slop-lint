# Release Process

This repository publishes two artifacts:

- GitHub Release archives used by `install.sh` and `install.ps1`.
- The Rust crate published to crates.io as `slop-lint`.

## Prerequisites

- A clean git working tree.
- Passing local checks:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo package
cargo publish --dry-run
```

- A configured crates.io token stored as the GitHub Actions secret
  `CARGO_REGISTRY_TOKEN` if the crate should be published by CI.

## Publish

1. Update `Cargo.toml` and `CHANGELOG.md`.
2. Commit the release.
3. Create and push a tag:

```sh
git tag v0.1.0
git push origin main --tags
```

The `Release` workflow builds platform archives and publishes the GitHub
Release. The `Publish crate` workflow publishes to crates.io when a `v*` tag is
pushed.

For a manual crates.io publish from a local machine, run:

```sh
cargo publish
```
