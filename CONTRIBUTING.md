# Contributing

Thanks for contributing to `tidy-json`.

## Setup

- Install stable Rust.
- Clone the repository.
- Run:

```sh
cargo build
cargo test
```

## Development workflow

- Keep changes focused and small.
- Add tests for behavior changes.
- Run lint checks before opening a PR:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Commits and PRs

- Use clear commit messages.
- Explain the user-facing impact in PR descriptions.
- Link related issues when possible.
