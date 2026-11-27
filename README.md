# Iconocrust

**A Rust-powered core platform for independent applications.**

Iconocrust is being built as a local-first system engine. The core owns shared system behavior; web, desktop, mobile, and CLI clients can be implemented in the languages best suited to each platform.

## Architecture

- **Core:** Rust application responsible for domain logic, runtime orchestration, storage, and security.
- **API:** Versioned HTTP + JSON boundary between the core and its clients.
- **Clients:** Independent applications that communicate with the core instead of duplicating its business rules or accessing its database directly.

See [the architecture notes](docs/architecture/README.md).

## Repository layout

```text
iconocrust/
├── .github/                 # CI workflows and repository automation
├── crates/
│   └── iconocrust-core/     # Rust core engine
├── docs/                    # Architecture and project documentation
├── Cargo.toml               # Rust workspace manifest
├── rust-toolchain.toml      # Rust toolchain configuration
└── README.md
```

## Getting started

Requires the stable Rust toolchain.

```bash
cargo run -p iconocrust-core
```

The initial core server binds to `127.0.0.1:7432`. Check its health endpoint:

```bash
curl http://127.0.0.1:7432/v1/health
```

## Development checks

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## License

Licensed under [Apache-2.0](LICENSE).
