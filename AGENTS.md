# Agent Instructions for Iconocrust

These instructions apply to the entire repository.

## Mandatory reading order

Before changing code, read:

1. `README.md`
2. `docs/architecture/CORE_SPEC.md`
3. `docs/architecture/README.md`
4. `.agents/skills/iconocrust-core-development/SKILL.md`
5. `docs/workflows/CORE_IMPLEMENTATION_WORKFLOW.md`

If a task is limited to a specific subsystem, also read its local documentation and all applicable nested `AGENTS.md` files.

## Authority and conflict resolution

Use this order when instructions conflict:

1. The user's latest explicit instruction.
2. Security and data-integrity requirements in `CORE_SPEC.md`.
3. This file.
4. The development skill and workflow.
5. Existing implementation conventions.

Do not silently resolve a material ambiguity. Identify it, explain the impact, and choose the safest reversible behavior only when work can reasonably proceed.

## Project invariants

- Iconocrust Core is the authoritative owner of business rules, validation, authorization, runtime lifecycle, and persistent data.
- Client applications communicate through documented, versioned APIs. They MUST NOT access Core-owned database files or tables directly.
- Keep the initial product as a modular Rust monolith. Do not split crates or processes without a concrete boundary and documented reason.
- Keep domain logic independent of Axum, HTTP types, database drivers, and UI frameworks.
- Keep API DTOs, domain types, and persistence models separate.
- HTTP + JSON is the default request/response transport. Use SSE for one-way streams when needed. Do not introduce WebSocket, gRPC, FFI, agents, model providers, or tool execution as speculative infrastructure.
- Bind to loopback by default. Do not enable non-loopback access without the security design and explicit user approval required by the Core specification.
- Do not add persistence or select a database without recording the required decision in an ADR first.
- Never commit secrets, credentials, local environment files, or generated private data.

## Implementation rules

- Work from the Core specification and explicit acceptance criteria, not assumptions.
- Inspect the existing code before editing. Preserve working behavior unless the task requires a change.
- Prefer the smallest complete change. Avoid unrelated refactors and speculative abstractions.
- Use stable Rust and the workspace toolchain. Respect the workspace edition, MSRV, formatting, and lint configuration.
- Add or update tests for every behavior change. Include negative/error-path tests where applicable.
- Update API and architecture documentation whenever externally observable behavior or a long-term design decision changes.
- Add dependencies only when necessary. Explain why a dependency is needed and avoid overlapping libraries.
- Do not suppress compiler or Clippy warnings merely to make CI pass. Fix the underlying issue or document a justified exception.
- Do not use `unwrap()` or `expect()` on runtime-controlled input or expected operational failures. Propagate typed errors.
- Do not expose internal errors, stack traces, secrets, filesystem paths, or environment values through API responses.
- Keep logs structured. Never log credentials, authorization headers, or sensitive user content by default.

## Required delivery process

For each task:

1. Identify the relevant specification sections and acceptance criteria.
2. Inspect relevant source files, tests, and existing patterns.
3. State a concise implementation plan when the task has multiple steps or meaningful design impact.
4. Implement the smallest coherent change.
5. Add/update tests and documentation.
6. Run the applicable checks, including the full workspace CI commands when feasible:
   - `cargo fmt --all -- --check`
   - `cargo check --workspace --all-targets`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo test --workspace`
7. Review the diff for unrelated changes, security issues, and missing tests.
8. Report changed files, behavior, test commands and actual results, and unresolved risks.

Never claim a command, test, build, or CI workflow passed unless its result was actually observed. If a command could not be run, state that explicitly and explain why.

## Definition of done

A task is complete only when:

- Its acceptance criteria are met.
- Relevant tests are added or updated.
- Relevant documentation is updated.
- Applicable formatting, check, lint, and test commands have been run or their inability to run is disclosed.
- No known security, compatibility, or data-integrity regression is left unexplained.
- The final report distinguishes implemented behavior from deferred work.
