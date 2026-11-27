# Iconocrust Core Specification

- **Status:** Baseline specification
- **Version:** 0.1.0
- **Applies to:** Iconocrust Core
- **Implementation language:** Rust
- **Normative terms:** MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY are used as requirement levels.

## 1. Purpose

Iconocrust is a local-first, API-driven core platform for applications that need a shared execution, persistence, and security layer. The Core is the authoritative owner of platform behavior. User-facing applications are clients of the Core; they are not alternate implementations of its business rules.

This document defines the initial system boundary and the contracts that implementation work must preserve. It is the baseline for code review, tests, and future architecture decisions.

## 2. Product goals

The Core MUST:

1. Expose a stable, documented API that clients written in different languages can consume.
2. Own domain rules, validation, authorization, runtime lifecycle, and persistence.
3. Provide predictable errors, structured logs, health information, and graceful shutdown.
4. Support local-first operation without requiring a hosted service.
5. Allow capabilities to be added behind explicit module boundaries without coupling clients to internal Rust types.
6. Keep compatibility concerns at the API boundary rather than exposing internal implementation details.

The initial implementation SHOULD favor a modular monolith. Internal modules may later become separate Rust crates or processes when there is a demonstrated need.

## 3. Non-goals for the initial milestone

The initial Core milestone does not implement:

- Agent planning or multi-agent orchestration.
- LLM provider integrations or model routing.
- Tool/plugin execution or a third-party extension sandbox.
- A remote hosted control plane, multi-tenant service, or cluster scheduler.
- A graphical user interface.
- A public internet-facing deployment profile.
- A general-purpose workflow engine.

These capabilities may be designed later, but MUST NOT be introduced as implicit responsibilities of the initial HTTP server.

## 4. System context and ownership

### 4.1 Core responsibilities

The Core owns:

- API routing and API-version compatibility.
- Request parsing, validation, and error translation.
- Application/domain services and business invariants.
- Authentication and authorization decisions.
- Runtime and background-task lifecycle.
- Persistence interfaces, migrations, and transaction boundaries.
- Configuration loading and validation.
- Operational health and observability.

### 4.2 Client responsibilities

A client application owns:

- User interface and presentation state.
- Platform-specific interaction and accessibility.
- Client-side convenience validation (never a replacement for Core validation).
- Calling the Core through its documented API.
- Rendering Core responses and errors.

A client MUST NOT access Core-owned database files directly, write to Core tables, or reimplement authoritative business rules.

### 4.3 Trust boundary

The Core is the trust boundary for platform data and operations. Client input is untrusted, including input from a local process. The Core MUST validate every request and enforce authorization at the operation boundary.

Internal Rust structs, database schemas, and module layouts are implementation details. They MUST NOT be treated as public API contracts.

## 5. Architecture

### 5.1 Initial deployment shape

The first release is one Rust process exposing an HTTP API. The process contains logically separated modules and may use multiple internal tasks, but it is deployed and versioned as one unit.

The initial transport choices are:

- HTTP + JSON for request/response APIs.
- Server-Sent Events (SSE) for one-way server-to-client event streams when required.
- WebSocket only when a feature requires bidirectional, low-latency communication that cannot be reasonably represented by HTTP or SSE.
- gRPC is deferred until there is a concrete cross-process or performance requirement.

The system MUST NOT require FFI between client applications and the Core.

### 5.2 Layering

The implementation SHOULD follow these dependency directions:

1. **API layer:** HTTP routes, DTOs, protocol-level validation, status codes.
2. **Application layer:** use cases, orchestration, authorization checks, transaction coordination.
3. **Domain layer:** entities, value objects, invariants, domain errors; independent of HTTP and database libraries.
4. **Infrastructure layer:** database adapters, filesystem access, clocks, process integrations, and external services.

Dependencies point inward. Domain logic MUST NOT depend on Axum, HTTP request types, SQL drivers, or a specific UI framework.

### 5.3 Module boundaries

The initial workspace may remain small. Add a module or crate when it has a clear responsibility and stable boundary, not merely to increase crate count.

Expected logical areas:

- `api`: versioned routes, request/response DTOs, protocol error mapping.
- `application`: use cases and orchestration.
- `domain`: core concepts and invariants.
- `config`: configuration types, loading, and validation.
- `observability`: tracing setup and operational signals.
- `storage`: persistence abstractions, adapters, and migrations.
- `runtime`: managed background tasks and lifecycle, when needed.

The current `iconocrust-core` crate may host these as modules initially. Extracting crates is a later refactor and MUST preserve public behavior.

## 6. API contract

### 6.1 General rules

- The API base path is `/v1`.
- JSON is the default representation.
- Field names use `snake_case`.
- Timestamps use RFC 3339 in UTC.
- Identifiers are opaque strings to clients; clients MUST NOT infer meaning from their format.
- The Core MUST reject unsupported content types with `415 Unsupported Media Type`.
- The Core MUST reject malformed JSON with `400 Bad Request`.
- Request and response DTOs are separate from domain and persistence models.
- Unknown request fields SHOULD be rejected for security-sensitive operations; otherwise behavior MUST be documented and tested.
- Every API change MUST update the API documentation and contract tests.

### 6.2 Versioning and compatibility

The URL major version changes only for breaking changes. Within `/v1`, additive optional response fields and new endpoints are generally compatible. A change that alters the meaning, type, requiredness, or behavior of an existing field is breaking.

The Core MUST NOT silently change the meaning of an existing field. Deprecated fields or endpoints SHOULD be documented with a migration path before removal. A breaking change requires a new major API version and a compatibility plan.

### 6.3 Initial endpoints

The initial service exposes:

| Method | Path | Purpose |
|---|---|---|
| GET | `/v1/health` | Liveness: process is responding |
| GET | `/v1/ready` | Readiness: required startup dependencies are usable |

Health endpoints MUST NOT disclose secrets, filesystem paths, environment values, or detailed internal errors. The readiness endpoint may return a non-success status while required dependencies are unavailable.

Future domain endpoints MUST be grouped by resource/use case and documented before implementation.

### 6.4 Success responses

A resource endpoint SHOULD return the resource representation directly unless an envelope is needed for pagination or metadata. Collection endpoints MUST define pagination semantics before they are introduced.

For asynchronous operations, the API MUST clearly distinguish accepted work from completed work. If an operation returns before completion, it SHOULD return `202 Accepted` with a stable operation identifier and a documented way to observe its state.

### 6.5 Error contract

All API errors use a consistent JSON shape:

```json
{
  "error": {
    "code": "validation_error",
    "message": "The request is invalid.",
    "request_id": "req_opaque_identifier",
    "details": []
  }
}
```

Requirements:

- `code` is a stable machine-readable identifier.
- `message` is safe for client display and MUST NOT expose secrets or internal stack traces.
- `request_id` correlates the response with server logs.
- `details` is optional structured context and MUST NOT contain credentials or sensitive values.
- Clients SHOULD branch on `code`, not parse the human-readable `message`.

The API MUST use HTTP status codes consistently: 400 malformed/invalid request, 401 unauthenticated, 403 unauthorized, 404 missing resource, 409 state conflict, 413 payload too large, 415 unsupported media type, 429 rate limited, 500 unexpected internal error, and 503 temporarily unavailable. More specific mappings MUST be documented.

Internal errors MUST be logged with diagnostic context, but stack traces and raw internal errors MUST NOT be returned to clients.

### 6.6 Request identity and limits

The Core MUST assign a request ID when one is not supplied. If a client-supplied correlation ID is supported, it MUST be validated, length-limited, and treated as untrusted metadata.

Request body size, header size, and request timeouts MUST have explicit configurable limits. Limits MUST have safe defaults and be covered by tests.

### 6.7 Streaming

SSE is the default for one-way progress and event delivery. Each event MUST have a documented event name and JSON data schema. Reconnection behavior and event retention/replay semantics MUST be explicitly defined by the feature that introduces the stream.

WebSocket MUST NOT be added solely as a generic future-proofing measure.

## 7. Configuration

Configuration is loaded once during startup unless a specific setting is explicitly designed for safe reload.

Requirements:

- Configuration MUST be represented by typed Rust structures.
- Invalid or contradictory configuration MUST fail startup with an actionable error.
- Secrets MUST NOT be committed to the repository or printed in logs.
- Defaults MUST be documented and safe for local development.
- Environment-variable names MUST use a consistent `ICONOCRUST_` prefix.
- Tests MUST be able to inject configuration without relying on the developer's machine environment.

Initial configuration surface:

| Setting | Default | Requirement |
|---|---|---|
| `ICONOCRUST_HOST` | `127.0.0.1` | Loopback-only by default |
| `ICONOCRUST_PORT` | `7432` | Must be a valid TCP port |
| `RUST_LOG` | `info` | Standard tracing filter |
| `ICONOCRUST_DATA_DIR` | Platform-appropriate application data directory | Must be writable when persistence is enabled |

The initial server MUST bind to loopback by default. Binding to a non-loopback interface MUST require an explicit configuration change and MUST NOT be considered safe until authentication and transport-security requirements for that deployment mode are implemented.

The exact cross-platform data-directory resolution and configuration-file format are implementation decisions that MUST be recorded in an ADR before persistence is introduced.

## 8. Persistence and data ownership

The Core is the sole writer and owner of its persistent data. Clients interact with data through API operations, never by opening the database directly.

Before persistent domain features are added, the implementation MUST define:

- The initial database engine and why it is appropriate.
- Database location and file-permission expectations.
- Schema migration strategy and migration ordering.
- Transaction boundaries and consistency guarantees.
- Backup, restore, and corruption-recovery expectations.
- Data deletion and retention behavior.
- Whether data is sensitive and how it is protected at rest.

Persistence models MUST remain separate from API DTOs and domain models. Schema changes MUST use versioned migrations. Tests MUST cover fresh database initialization and upgrades from the previous supported schema.

No domain data schema is prescribed by this baseline document; it must be specified by the feature that owns the data.

## 9. Security requirements

Security is a Core responsibility, not a client responsibility.

Minimum requirements:

1. Bind to loopback by default.
2. Treat all client input as untrusted.
3. Validate identifiers, paths, sizes, enum values, and state transitions at the Core boundary.
4. Do not expose secrets, tokens, environment contents, or internal stack traces in API responses or logs.
5. Use least privilege for filesystem and process access.
6. Avoid arbitrary command execution and arbitrary filesystem access in generic API handlers.
7. Apply authentication and authorization before enabling non-loopback access or sensitive operations.
8. Keep credentials out of source control and test fixtures.
9. Add tests for unauthorized access and common input-validation failures as protected operations are introduced.

CORS MUST be restrictive and documented if browser clients are supported. CORS is not authentication and MUST NOT be treated as an authorization control.

A threat model and an authentication ADR are required before remote access, multi-user access, or sensitive tool execution is implemented.

## 10. Runtime lifecycle

The Core process MUST have explicit startup and shutdown phases.

Startup sequence SHOULD:

1. Load and validate configuration.
2. Initialize structured logging.
3. Initialize required infrastructure and storage.
4. Run or verify required migrations.
5. Construct application state.
6. Bind the listener.
7. Report readiness only after required dependencies are usable.

Shutdown MUST:

- Stop accepting new work.
- Allow in-flight requests a bounded grace period to complete.
- Signal managed background tasks to stop.
- Close storage and other resources cleanly.
- Exit with an appropriate process status.

Background tasks MUST be owned by a lifecycle manager or equivalent application state. Detached, untracked tasks SHOULD NOT be used for work that must complete or be cancelled reliably.

## 11. Observability and diagnostics

The Core MUST use structured tracing rather than ad hoc print statements.

Logs SHOULD include:

- Timestamp and severity.
- Service name and version.
- Request ID where applicable.
- Operation or route name.
- Outcome and duration.
- Error code for failed operations.

Logs MUST NOT include passwords, API keys, access tokens, raw authorization headers, or sensitive user content by default.

The Core SHOULD expose liveness and readiness separately. Metrics are deferred until there is a concrete operational need, but instrumentation SHOULD avoid designs that make metrics difficult to add later.

## 12. Error handling inside the Rust application

Domain and application errors MUST be represented as typed errors. HTTP status mapping belongs at the API boundary.

Requirements:

- Do not use string matching on error messages to determine behavior.
- Do not leak infrastructure-specific error types through the public API.
- Preserve causal error context for internal diagnostics.
- Convert errors at explicit architectural boundaries.
- Use `Result` for recoverable failures; panics are reserved for violated programmer invariants and MUST NOT be used for expected input or infrastructure failures.

## 13. Testing and quality gates

Every feature MUST include tests appropriate to its boundary.

Required test layers:

- **Unit tests:** domain invariants, configuration validation, error mapping.
- **API tests:** routes, status codes, JSON schemas, malformed input, request IDs.
- **Integration tests:** storage behavior and migrations when persistence exists.
- **Lifecycle tests:** startup failure and graceful shutdown behavior where practical.
- **Compatibility tests:** API contract behavior for supported versions.

The repository CI MUST run:

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

A feature is not complete merely because it compiles. Its acceptance criteria and tests must pass.

## 14. Repository and documentation conventions

- Architecture decisions with long-term consequences MUST be recorded as ADRs under `docs/architecture/decisions/`.
- API contracts MUST be documented under `docs/api/` before client integration.
- Each module MUST have a clear responsibility and avoid circular dependencies.
- Public Rust APIs MUST have useful documentation comments.
- Examples MUST be executable or clearly labeled as illustrative.
- Documentation changes MUST accompany behavior and contract changes.

## 15. Delivery milestones

### Milestone 0 — Core foundation

- Workspace and executable compile.
- Configuration is typed and validated.
- Structured logging is initialized.
- HTTP server starts and shuts down gracefully.
- Liveness and readiness endpoints are defined and tested.
- API error response and request ID behavior are implemented.
- Loopback binding is the default.

### Milestone 1 — Persistence foundation

- Database choice and data-directory behavior are recorded in ADRs.
- Storage abstraction and migration mechanism exist.
- Fresh initialization and migration tests pass.
- Core remains the only database owner.

### Milestone 2 — First domain capability

- One concrete domain use case is specified end-to-end.
- Domain rules and application service are independent of HTTP and storage libraries.
- API request/response and error contracts are documented.
- Unit, API, and integration tests cover acceptance criteria.

Agent orchestration, model providers, and tool execution are considered only after these foundations are stable.

## 16. Acceptance criteria for this specification

This specification is considered implemented incrementally when:

- [ ] The Core has a documented and enforceable boundary from client applications.
- [ ] API DTOs are separate from domain and persistence models.
- [ ] The server binds to loopback by default.
- [ ] Configuration is typed, validated, and documented.
- [ ] Request IDs are available in API errors and structured logs.
- [ ] Error responses follow the documented contract.
- [ ] Liveness and readiness have distinct semantics.
- [ ] Startup and shutdown manage resources and background tasks.
- [ ] Persistence decisions and migrations are documented before domain data is stored.
- [ ] CI enforces formatting, compilation, linting, and tests.
- [ ] Each implemented milestone has tests demonstrating its acceptance criteria.

This checklist is a roadmap; unchecked items are not claims that the current code already implements the requirement.

## 17. Open decisions

The following decisions are intentionally not made by this baseline and require ADRs before the corresponding implementation:

1. Database engine and migration library.
2. Cross-platform application data-directory resolution.
3. Configuration file format and precedence rules.
4. Authentication mechanism for non-loopback or multi-user deployments.
5. API schema publication format (for example, OpenAPI).
6. Event persistence and replay semantics for SSE.
7. Backup, restore, and data-retention policy.

Until an ADR resolves an item, implementation MUST NOT silently assume a choice that changes the security, compatibility, or data-integrity model.
