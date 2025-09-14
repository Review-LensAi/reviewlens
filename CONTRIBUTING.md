# Contributing

Thank you for considering contributing to this project! We welcome issues and pull requests from the community.

## Development Workflow

1. Fork the repository and create your branch from `main`.
2. If you have added code, add tests.
3. Ensure all tests pass with `cargo test`.
4. Run `cargo fmt` before committing.
5. Submit a pull request with a clear description of your changes.

## Rust Toolchain

This project pins its Rust version using `rust-toolchain.toml`. The current
required version is Rust 1.82. Install and select this toolchain with:

```bash
rustup toolchain install 1.82
rustup default 1.82
```

Using the pinned toolchain keeps builds consistent with the version enforced
in continuous integration.

## Reporting Issues

Please use the GitHub issue tracker to report bugs or request features. Provide as much detail as possible to help us reproduce the problem or understand your idea.

## Code Review

All submissions require review. Please be responsive to feedback and update your pull request accordingly.

## License

By contributing, you agree that your contributions will be licensed under the Apache-2.0 license.
