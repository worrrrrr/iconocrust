# Architecture

Iconocrust is designed as a Rust-owned core with language-independent application clients.

## Initial boundary

- The core owns domain rules, persistence, runtime orchestration, and security decisions.
- Clients communicate with the core through versioned HTTP APIs using JSON.
- Streaming will use Server-Sent Events where one-way event delivery is sufficient.
- Applications must not access the core database directly.

## Initial implementation

The first milestone is a single Rust crate exposing `GET /v1/health` on `127.0.0.1:7432`. Keep the implementation modular inside the crate until domain boundaries justify splitting it into more crates.
