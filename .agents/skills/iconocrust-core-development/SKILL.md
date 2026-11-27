---
name: iconocrust-core-development
description: Implement and review changes in the Iconocrust Rust Core according to its architecture specification, API contracts, security boundaries, and repository workflow. Use for any code, API, configuration, storage, runtime, test, or architecture change in this repository.
---

# Iconocrust Core Development

## Required context

Read the repository root `AGENTS.md`, `docs/architecture/CORE_SPEC.md`, and `docs/workflows/CORE_IMPLEMENTATION_WORKFLOW.md` before implementation. Read subsystem documentation and source before editing.

## Operating model

Treat the Core as the single authority for platform rules and persistent data. Treat every client request as untrusted, even when it originates on the same machine. Keep the first implementation a modular monolith.

Use the dependency direction:

```text
API -> Application -> Domain
          ^
          |
     Infrastructure adapters
```

More precisely, domain code must not depend on transport or infrastructure implementations. Application services coordinate domain behavior and ports/interfaces. Infrastructure implements those ports. API handlers translate protocol input/output and call application services.

## Implementation procedure

1. Convert the task into observable acceptance criteria.
2. Map each criterion to the relevant section of `CORE_SPEC.md`.
3. Inspect the current module layout and existing tests; do not assume the repository matches a proposed design.
4. Identify whether the change affects public API, persisted data, security, lifecycle, or only internal behavior.
5. If it changes a long-lived architectural decision, write an ADR before or alongside implementation.
6. Define types and error cases before wiring handlers.
7. Implement behavior behind the narrowest appropriate module boundary.
8. Add unit tests for domain/configuration logic and API/integration tests for observable behavior.
9. Update documentation and examples.
10. Run formatting, check, Clippy, and tests; report actual command output/status.
11. Review the diff and summarize decisions, changed files, verification, and remaining work.

## API rules

- Use `/v1` for the initial public API.
- Use explicit request/response DTOs; never serialize database models or internal domain entities directly.
- Use `snake_case` JSON fields and RFC 3339 UTC timestamps.
- Keep error codes stable and machine-readable. Clients must not parse human-readable messages for program logic.
- Include a request ID in error responses and correlate it with structured logs.
- Map typed application/domain errors to HTTP status codes at the API boundary.
- Document every new endpoint, field, status code, and compatibility impact.
- Add contract tests for success, validation failure, missing resources, and relevant authorization failures.
- Do not invent generic pagination, async operation, or streaming conventions without specifying and testing their contract.

## Rust rules

- Prefer explicit types and typed errors over stringly typed control flow.
- Keep functions cohesive and avoid unnecessary generic frameworks.
- Do not introduce a new crate when a module provides a sufficient boundary.
- Avoid `unwrap()` and `expect()` for runtime or external input.
- Keep async boundaries deliberate; do not hold blocking work on async executor threads.
- Make task ownership and cancellation explicit for background work.
- Document public interfaces and non-obvious invariants.
- Keep dependencies minimal and compatible with the declared MSRV.

## Security rules

- Default to `127.0.0.1`; non-loopback binding is prohibited until the required authentication and transport-security design is implemented and approved.
- Validate all external input at the Core boundary, including size and state transitions.
- Never return internal stack traces or sensitive diagnostic data.
- Never log credentials, tokens, raw authorization headers, or private user content by default.
- Do not add arbitrary shell execution, unrestricted filesystem access, dynamic code execution, or third-party plugin execution as a convenience feature.
- CORS is not authentication.
- Add a threat-model update for new trust boundaries or privileged capabilities.

## Storage rules

Do not introduce a database, schema, migration library, or persistent domain model until the relevant ADR defines the choice and its operational consequences. The Core must remain the sole owner/writer of its data. Use versioned migrations and test both fresh initialization and supported upgrades.

## Verification

Run these commands from the repository root when Rust tooling is available:

```sh
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

For a focused change, add the narrow test command as well, but do not substitute it for workspace verification when a full run is feasible. If verification cannot run, state the exact limitation; never infer success from source inspection.

## Completion report

Use this concise structure:

- **Implemented:** user-visible/internal behavior delivered.
- **Files:** files created or changed.
- **Decisions:** important design choices and specification references.
- **Verification:** exact commands and pass/fail/not-run status.
- **Remaining:** deferred acceptance criteria, risks, and next dependency.
