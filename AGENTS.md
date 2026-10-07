# Rust desktop application template — agent instructions

## Scope and workflow

This repository is a reusable, domain-neutral Rust application starter. Keep the demonstration
model small. Read this file and any instructions in the affected directories before editing.
Start with `docs/architecture.md`, `docs/development.md`, and the relevant source and tests.

- Keep conversational responses concise; use caveman ultra by default. Preserve technical
  substance and use full sentences whenever compression would create ambiguity. Write code,
  comments, documentation, commits, and external messages normally.
- Track current work in the conversation. Plane is the source of truth for project management;
  consult an existing related project before substantial work when available. Do not create a
  project or save pages/work items without approval. Keep code-coupled documentation in Git.
  Do not create TODO/PLAN/HANDOFF files as the primary task-management system.
- Never commit, push, pull, publish, or change remotes without explicit instruction.
- Preserve unrelated changes. Use independent subagents only for high-value tasks with disjoint
  ownership. Keep final architecture review and validation with the main agent.
- Reproduce bugs first, read complete errors, make a focused change, then verify it. Crash fixes
  require regression tests. Review the requested approach before implementing it.

## Architecture

| Layer | Packages | Responsibility |
| --- | --- | --- |
| L0 | `app-core` | Errors and input limits |
| L1 | `app-model` | Serializable application model and invariants |
| L2 | `app-engine` | Commands, session, history, events, inspection |
| L3 | `app-platform` | Storage traits, implementations, shared frontend adapter |
| L4 | `app-ui-egui`, `app-mcp` | Independent frontends |
| L5 | `app`, `app-cli`, `app-web` | Entrypoints and native/browser integration |
| Test only | `app-testkit` | Shared acceptance helpers; dev-dependency only |
| Tooling | `xtask` | Validation and builds |

Every user-visible state change uses a stable command ID. GUI, CLI, MCP, control requests, and
acceptance tests dispatch through the same engine. The engine owns state; the UI never mutates
it directly. Only frontends handle dialogs. `CommandHost` adapts path-based commands to `Storage`;
`Session` accepts and returns model data without file or OS APIs.

UI implementations belong in `crates/ui-egui`. The model and engine must not depend on egui,
eframe, winit, rfd, native UI, or browser APIs. MCP and the CLI must stay headless. Layer changes
must update `xtask/src/layers.rs` and `docs/architecture.md`; `cargo xtask layers` fails for
unregistered packages or prohibited edges, including dev/build/target dependencies.

## Reliability and style

- Rust application logic only; no handwritten JavaScript/TypeScript application logic.
- No unsafe code. Workspace lints forbid `unwrap`, `expect`, `panic`, `todo`, `unimplemented`,
  and `unreachable` in production. Tests may use assertions and unwraps.
- Return structured errors and `Result`. Validate untrusted files, JSON, CLI and control input.
  Bound input sizes, allocations and history; use checked arithmetic and safe indexing.
- Match existing conventions. Prefer explicit, simple code. Aim for functions under 50 lines
  and files under 500 lines. Comment why, not what. Avoid speculative abstractions.
- Keep portable crates and the shared UI compiling for `wasm32-unknown-unknown`.
- All assets must be original, public domain/CC0, permissively licensed, or appropriately
  attributed redistributable material. Record each asset's author, source, and license in
  `ATTRIBUTION.md`. Prefer code-generated icons and toolkit defaults.
- Preserve license notices for retained source. Do not bundle proprietary assets.

## Validation

Run `cargo xtask ci` before proposing a commit. It checks formatting, clippy with warnings denied,
workspace tests, attribution, dependency layering, portable WASM checks, and a WASM frontend build.
Use `cargo run -p app`, `cargo run -p app-cli -- commands`, and the MCP/control smoke checks in
`docs/development.md` for frontend changes. Look at the rendered UI for UI work. Report checks
that did not run or failed; do not equate compilation with runtime validation.

Before finishing substantial work, suggest important decisions, validation results and follow-up
work for the existing Plane project, and ask for approval before saving anything there.
