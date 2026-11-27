# Iconocrust Core — Progress

## Current status

- Milestone 0 (Core foundation): **in progress** — all §15 checklist items for this slice are
  implemented, covered by tests, and verified against the running binary; remaining Milestone 0
  scope outside this slice (see "Next task") is not started.
- Workspace compiles; single crate `iconocrust-core`
- Implemented: `GET /v1/health` (liveness, 200, JSON `{status, service}`)
- Implemented: `GET /v1/ready` (readiness, 200/503)
- Implemented: typed `Config` with env vars `ICONOCRUST_HOST`, `ICONOCRUST_PORT`, `ICONOCRUST_DATA_DIR`
- Implemented: error envelope `{error:{code,message,request_id,details}}`
- Implemented: request ID via tower-http — `SetRequestIdLayer` (client `x-request-id` honored when
  present and valid, otherwise UUID v4) paired with `PropagateRequestIdLayer`, so the ID appears on
  every response
- Implemented: error responses carry the request's `x-request-id` (fallback 404/405 included), so
  bodies, headers, and server logs share one correlation ID (CORE_SPEC §6.5)
- Implemented: graceful shutdown (SIGTERM/SIGINT)
- Loopback bind `127.0.0.1:7432` (configurable via env); IPv4 and IPv6 hosts both supported
- Basic tracing via `tracing_subscriber` with `RUST_LOG` (default `info`)
- Structured startup logs with service+version
- No persistence, no domain capability (correctly deferred)

## Milestone 0 checklist (CORE_SPEC §15)

- [x] Workspace and executable compile
- [x] Configuration typed and validated (IPv4/IPv6 loopback and non-loopback covered)
- [x] Structured logging initialized (basic)
- [x] HTTP server starts and shuts down gracefully
- [x] Liveness and readiness endpoints defined and tested
- [x] API error responses returned in the §6.5 envelope on every error path
- [x] Request ID set + propagated and correlated into error bodies and logs
- [x] Loopback binding is default
- [x] Unit tests for config validation, error mapping, ready status
- [x] Router-level API tests (`crates/iconocrust-core/tests/api_routes.rs`) for `/v1/health`,
      `/v1/ready`, request ID propagation (generated + client-supplied), 404/405 envelopes, and
      `ApiError` → HTTP status/envelope mapping
- [x] `cargo fmt/check/clippy/test --workspace` pass (29 tests)

## Reviewer follow-up fixes (round 1) — DONE

- **IPv6 loopback** (`config.rs`): `socket_addr()` builds `SocketAddr::new(ip, port)` instead of
  parsing `"host:port"`, so `::1` no longer panics. `is_loopback()` uses `IpAddr::is_loopback()`.
  Verified running the binary with `ICONOCRUST_HOST=::1` → listened on `[::1]:7533`, `/v1/health`
  returned 200.
- **Request ID propagation** (`api/mod.rs`): `SetRequestIdLayer` is now paired with
  `PropagateRequestIdLayer`; `MakeRequestUuid` reuses a client-provided `x-request-id`.
- **Error correlation** (`api/error.rs`): `ApiError::to_envelope(request_id)` takes the caller's ID;
  `error_handling_middleware` renders empty-body 404/405 responses as §6.5 envelopes using the
  request's `x-request-id` and logs failures with the same ID. `IntoResponse` remains for
  out-of-router use and generates a UUID only when no request context exists.
- **Router-level API tests**: `crates/iconocrust-core/tests/api_routes.rs` (9 tests) plus 11
  in-module router tests.

Files touched in this round:
- `crates/iconocrust-core/src/config.rs` — IPv6-safe `socket_addr`/`is_loopback` (+3 unit tests)
- `crates/iconocrust-core/src/api/mod.rs` — set/propagate pairing, middleware wiring, router tests
- `crates/iconocrust-core/src/api/error.rs` — request-ID-correlated envelopes + correlation middleware
- `crates/iconocrust-core/tests/api_routes.rs` (new) — router-level API tests
- `crates/iconocrust-core/Cargo.toml` — dev-deps `http-body-util`, `tower` (`util`)

Spec refs: CORE_SPEC §6.3, §6.5, §6.6, §7, §10, §13, §15.

@Reviewer ready for re-review.

## Next task for Coder

**Task:** Milestone 0 vertical slice — typed configuration, `/v1/ready`, request ID in error
responses. — **DONE** (including Reviewer round-1 fixes)

Files created/modified for that slice:
- `crates/iconocrust-core/src/config.rs` (new) — typed `Config` struct
- `crates/iconocrust-core/src/api/error.rs` (new) — error envelope
- `crates/iconocrust-core/src/api/ready.rs` (new) — `GET /v1/ready`
- `crates/iconocrust-core/src/api/mod.rs` — routes, request ID layers, error middleware
- `crates/iconocrust-core/src/main.rs` — config loading, graceful shutdown, structured logs
- `crates/iconocrust-core/src/lib.rs` — added config module
- `crates/iconocrust-core/Cargo.toml` — tower-http, uuid, thiserror, serde_json; dev-deps for tests

**Awaiting Architect** for the next slice.
