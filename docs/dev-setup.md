## Development assumptions

- Use the stable Rust toolchain from `rust-toolchain.toml`
- Format with `cargo fmt`
- Lint with `cargo clippy --workspace --all-targets`
- Test with `cargo test --workspace`
- Do not introduce nightly-only features unless explicitly requested
- Prefer adding small, composable crates/modules over monolithic files
