# Use the template

## 1. Create a project

Use your Git host's template action or clone/copy this repository. Select your own remote and
license policy; retain notices required by the existing MIT/Apache source. Do not publish or
change remotes as an incidental part of renaming.

## 2. Rename packages and entrypoints

The working directory name is independent of Cargo package names. For a project called `onecad`:

| Template package | Example name |
| --- | --- |
| `app-core` | `onecad-core` |
| `app-model` | `onecad-model` |
| `app-engine` | `onecad-engine` |
| `app-platform` | `onecad-platform` |
| `app-ui-egui` | `onecad-ui-egui` |
| `app-mcp` | `onecad-mcp` |
| `app-testkit` | `onecad-testkit` |
| `app`, `app-cli`, `app-web` | `onecad`, `onecad-cli`, `onecad-web` |

Update these locations together:

1. Root `Cargo.toml`: workspace dependency keys/paths and default member paths.
2. Each package `Cargo.toml`: package name and dependency references.
3. Rust imports: `app_engine` becomes `onecad_engine`, and likewise for other packages.
4. Binary tests: `CARGO_BIN_EXE_app-cli` becomes `CARGO_BIN_EXE_onecad-cli`.
5. `xtask/src/layers.rs`: package registrations, allowed edges and frontend restrictions.
6. `xtask/src/main.rs`: WASM package names and build command.
7. `.github/workflows/`, `packaging/`, web title, native title, MCP server name, temporary-file
   prefix, documentation and example paths/commands.
8. Rename `apps/app*` directories if desired and update workspace paths and `Trunk.toml` paths.

Use a reviewed replacement of exact package names/imports; do not replace the bare word `app`
globally, which also appears in code concepts and third-party APIs. Regenerate `Cargo.lock` with
`cargo check --workspace`, then search for leftover `app-template`, `app-core`, `app_engine`,
`CARGO_BIN_EXE_app-cli`, and old packaging identifiers. Keep Cargo.lock for reproducible apps.

## 3. Replace the model

Replace `AppModel` and its validation in `crates/model`. Decide which state is persisted and
which is presentation-only. If the file schema changes, add a format version and migration
rules before distributing files to users. Undo uses bounded snapshots; larger models may need
shared immutable data or edit deltas without changing the frontend command contract.

## 4. Add domain crates only as needed

Create real domain functionality below the engine. Register each package and allowed dependency
edges in xtask, update architecture docs, and test without a GUI. Do not create placeholder
crates. Keep domain types independent from rendering/window/transport types.

## 5. Add commands

Add command metadata, typed parameters, dispatch and tests as described in [commands](commands.md).
Keep IDs stable after exposing them to users or agents. Extend the shared host for file/service
interactions; never copy state mutation logic into individual frontends. Remove demo commands
when real commands replace them.

## 6. Customize the UI and services

Change `crates/ui-egui` panels, menus, tokens and widgets. Inject concrete services from the
entrypoint. Replace the UI entirely if another native toolkit suits the project; retain the
engine/command contracts. Implement persistent browser storage if retaining the web frontend.
If removing web support, remove its app, workspace/xtask registrations, workflow and docs.

## 7. Customize packaging

Set names, reverse-domain application IDs, version policy and platform metadata in `packaging`.
Add original or permissively licensed assets and attribution rows. Configure signing and
release publication explicitly; they are not enabled by this template. Validate actual
installations on each target OS before promising support.

## 8. Verify

Run `cargo xtask ci`, desktop/CLI/MCP smoke checks, and browser checks if retained. Test save/open,
undo/redo, errors and hostile files. Keep code-coupled docs in Git; link them from your existing
project management system rather than duplicating them.
