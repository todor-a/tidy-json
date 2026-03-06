# Release process

`tidy-json` uses two workflows:

- `release-plz.yml` to prepare release PRs and changelog/version updates.
- `release.yml` (cargo-dist) to build artifacts and publish releases/tap/npm packages after tags are pushed.

In short:

1. Merge release-plz PR to `main`.
2. Push or create the release tag produced by that PR.
3. Let `release.yml` build and publish binaries/installers.
