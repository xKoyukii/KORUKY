# Contributing to KORUKY

Thanks for considering a contribution.

## Before you start

- Check existing issues before opening a new one.
- Keep pull requests focused and explain the motivation.
- Do not add unofficial scraping, credential extraction, cookie handling, or
  subscription/access-control bypasses.
- Mark planned or mocked functionality clearly.

## Development checks

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Pull requests

Include:
- A concise description of the change.
- Tests for new behavior where practical.
- Documentation updates for user-visible changes.
- Screenshots or terminal recordings for TUI changes, when useful.
