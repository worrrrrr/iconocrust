# Core Implementation Workflow

This workflow governs how an agent turns Iconocrust requirements into reviewed, testable repository changes.

## 0. Intake and scope

- Read root `AGENTS.md`, the Core specification, the development skill, and relevant subsystem docs.
- Restate the requested outcome as observable behavior.
- Identify the affected subsystem and specification sections.
- Separate required work from optional improvements.
- Identify whether the task touches public API, security, persistence, configuration, runtime lifecycle, or compatibility.

**Gate:** Do not code while a material requirement is ambiguous. Ask one focused question only when the ambiguity cannot be resolved from repository context or a safe reversible default.

## 1. Repository reconnaissance

- Inspect the relevant source, tests, Cargo manifests, CI, and documentation.
- Trace the current request/data flow before changing it.
- Check for existing utilities or dependencies that already solve the problem.
- Check current git diff/status when operating in a working tree; do not overwrite unrelated user changes.
- Record current behavior and any known gap.

**Output:** short findings and a minimal implementation plan for non-trivial work.

## 2. Design

- Define acceptance criteria before implementation.
- Choose the narrowest module boundary that owns the behavior.
- Keep domain rules transport- and storage-independent.
- Define typed inputs, outputs, and error cases.
- For public API changes, define method/path, request and response schemas, status codes, error codes, and compatibility impact.
- For persistence changes, write an ADR first covering database choice, location, migrations, transactions, backup/restore, and data sensitivity.
- For new trust boundaries or privileged operations, update the threat model and security design first.
- Avoid speculative frameworks and dependencies.

**Gate:** Design must comply with `docs/architecture/CORE_SPEC.md`. If it does not, update the specification/ADR through an explicit decision rather than silently drifting.

## 3. Implementation

- Implement one coherent vertical slice at a time.
- Keep changes scoped; do not bundle unrelated cleanup.
- Keep API DTOs, domain models, and persistence models distinct.
- Keep error conversion at architectural boundaries.
- Keep configuration typed, validated, documented, and injectable in tests.
- Keep startup/shutdown ownership explicit for resources and background tasks.
- Update docs alongside observable behavior.

## 4. Testing

Add tests at the boundary where the behavior can fail:

| Change | Minimum test expectation |
|---|---|
| Domain rule | Unit tests for valid, invalid, and boundary cases |
| Configuration | Defaults, valid overrides, invalid values, missing required values |
| API endpoint | Success schema/status, malformed input, validation errors, request ID |
| Error mapping | Typed error to stable code and HTTP status |
| Storage | Fresh initialization, CRUD invariants, transaction behavior, migration upgrade |
| Security | Unauthorized access, invalid input, boundary/size limits, secret redaction |
| Runtime | Startup failure, readiness transition, graceful shutdown where practical |
| Compatibility | Existing contract behavior remains unchanged |

Tests must be deterministic and must not depend on developer-specific machine state, live external services, or real credentials unless the test is explicitly marked as an integration test with controlled prerequisites.

## 5. Local quality checks

Run from repository root:

```sh
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Fix failures caused by the change. Do not suppress warnings without a documented reason. If a check cannot run, record the missing tool, environment constraint, or external dependency. Do not report unrun checks as passed.

## 6. Review

Review the complete diff and verify:

- No unrelated files or generated artifacts were changed.
- No secret or local machine data was added.
- No API contract changed accidentally.
- No internal error or sensitive data leaks through logs/responses.
- No client-side assumption bypasses Core validation or authorization.
- No new dependency or abstraction lacks a concrete need.
- Documentation and tests match the implemented behavior.
- All acceptance criteria are either met or explicitly deferred.

## 7. Delivery

The final task report must include:

1. What changed and why.
2. Exact files changed.
3. Relevant specification/ADR sections.
4. Commands actually run and their observed results.
5. Acceptance criteria not met and why.
6. Risks or follow-up work.

If changes are committed or pushed, report the resulting commit/PR reference. Never imply that code was executed or CI passed based only on reading or writing files.

## 8. Milestone sequencing

Follow the baseline milestones in `CORE_SPEC.md`:

1. Core foundation: typed configuration, structured logging, lifecycle, health/readiness, request IDs, error contract, loopback binding.
2. Persistence foundation: ADR, data directory, storage adapter, migrations, initialization/upgrade tests.
3. First domain capability: end-to-end domain use case, documented API, unit/API/integration tests.
4. Only after those foundations are stable, consider agent orchestration, model providers, and tool execution.

Do not skip a milestone because a later capability appears more interesting.
