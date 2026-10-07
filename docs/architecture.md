# Architecture

The engine and model are independent of windows, operating systems and transports. The same
command changes the same state whether called by a person, a script, a test, or an agent.

```text
              Desktop UI
                   |
Web UI -------- ui-egui
                   |
CLI -------- CommandHost ------- MCP / Agent
                   |   |
                   |   Storage implementation
                   |
                 ENGINE
                   |
                 MODEL
                   |
                  CORE
```

## Crate responsibilities

- `app-core` (L0): structured `AppError` and shared size/history limits.
- `app-model` (L1): `AppModel { name: String, counter: i64 }`, serialization and invariants.
- `app-engine` (L2): `Session`, command metadata, validation, mutation, events, inspection and
  bounded undo/redo. It has no file, browser, UI or window APIs.
- `app-platform` (L3): `Storage` and `FileDialogs` service traits, native/memory storage and
  `CommandHost`. It depends on the engine; the engine never depends on it.
- `app-ui-egui` and `app-mcp` (L4): independent adapters. Neither depends on the other.
- `app`, `app-cli`, `app-web` (L5): entrypoints and concrete native/browser integration.
- `app-testkit`: dev-only consumer helpers. It can use headless services without a window.
- `xtask`: workspace tooling, not an application library.

`cargo xtask layers` reads Cargo metadata and enforces explicit allowed workspace edges. It also
rejects UI/browser dependencies in core/model/engine/platform/MCP/CLI and checks dev, build,
and target-specific declarations. Unknown crates fail until their rules are registered. The
CLI cannot depend on the UI; lower crates cannot depend on entrypoints or tooling. The UI owns
the substantive egui implementation; native entrypoints only wrap it and provide services.

Add domain crates only when a real application needs them. Place domain types and operations
below the engine, register their allowed edges in `xtask/src/layers.rs`, and keep their interfaces
free of frontend types. A new UI can depend on engine/platform and replace `ui-egui` completely.

## Commands and state

`Session::execute(id, params)` validates input, calls a registered handler, and returns
`CommandResult { state, events, data }`. `state` is an inspection snapshot with the model,
undo/redo availability and presentation flags; `data` carries command output. Model fields are
read-only to callers through `Session::model()`. The command registry supplies labels, parameter
schemas and undoability for discovery; enablement is evaluated against the current session.

Successful demo edits store the previous model, clear redo, and emit state/history events.
No-op edits preserve redo. Each history stack retains at most 100 snapshots. New/open commands
reset history, so undo never crosses an application-state load. Panel visibility is session
presentation state, outside persisted model/history. Error paths preserve state and history.

To add a command, register metadata and its handler in `engine/src/commands.rs`, implement it in
`engine/src/session.rs`, and behavior/hostile-input tests. This intentionally uses a compile-time
registry rather than a plugin framework. Stable IDs decouple frontends from implementations.

## Persistence boundary

The pure engine opens `{ "data": { "name": "Example", "counter": 5 } }` and saves its model to
`CommandResult.data`. Every frontend uses `CommandHost::execute`, which additionally accepts
`{ "path": "example.json" }` for the same open/save command IDs. The host reads/parses or
serializes/writes through `Storage`; it never duplicates model changes or validation.

`NativeStorage` bounds reads to 1 MiB plus one detection byte. Saves write and flush a uniquely
reserved sibling temporary file before renaming it over the destination; failed saves preserve
session state and do not truncate the previous file. The file's containing directory must exist.
On Unix, new files start with mode `0600`; overwrites preserve the destination's permissions.
This is not a full crash-recovery or autosave system, and directory durability is not promised.

`MemoryStorage` bounds files, per-file sizes and aggregate bytes. It is useful for tests and the
web demonstration. Provide another `Storage` implementation for browser persistence or your
application's infrastructure. Frontends inject `FileDialogs`; headless execution never opens
a dialog. Add clipboard, notifications, directories or URL services only when needed.

## Reliability

All template packages inherit `unsafe_code = "forbid"` and deny panic-producing shortcuts in
production. JSON must have the expected shape; unknown fields fail. Names are limited to 256
UTF-8 bytes, counters use checked signed 64-bit arithmetic, JSON input is bounded, and nesting
is limited. TCP/MCP frame limits are enforced before retaining unbounded input. Boundary tests
cover overflow, malformed state, history branching, failed storage and oversized frames.
