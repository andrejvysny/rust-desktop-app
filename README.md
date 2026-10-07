# Rust desktop application template

A small, engine-first Rust workspace for native applications. It provides an egui/eframe desktop
UI, an optional Rust/WASM frontend, a headless CLI, MCP agent tools, and a local control API.
Every interface uses the same command IDs, model validation, and undo/redo implementation.
There is no webview runtime. The browser frontend compiles the same Rust UI to WebAssembly.

The example state is only a name and counter. Replace it with your application model; the
surrounding command, storage, testing and frontend boundaries remain reusable.

```text
Desktop / Web ── ui-egui ─┐
CLI ─────────────────────┼── CommandHost ── engine ── model ── core
MCP / agent ─────────────┘        │
                              Storage
```

## Run

Requires Rust 1.95 or newer and native graphics support. Linux build prerequisites are listed
in [development](docs/development.md).

```sh
cargo run -p app
cargo run -p app-cli -- commands
cargo run -p app-cli -- run demo.incrementCounter
cargo run -p app-cli -- run demo.setName '{"name":"Example"}' --save example.json
cargo run -p app-cli -- state --load example.json
cargo run -p app-cli -- mcp
```

The native window has a menu, a name field, increment/decrement buttons, undo/redo, JSON file
open/save, and an optional state panel. CLI processes start fresh; use `--load`/`--save` for
persistence or MCP for a persistent headless session.

## Workspace

| Path | Purpose |
| --- | --- |
| `crates/core` | Structured errors and input limits |
| `crates/model` | Tiny serializable `AppModel` |
| `crates/engine` | Session, registry, commands, bounded history and events |
| `crates/platform` | Storage services and shared frontend command adapter |
| `crates/ui-egui` | Swappable, thin egui UI |
| `crates/mcp` | JSON-RPC MCP server: inspect, execute, inspect again |
| `crates/testkit` | Reusable acceptance test helpers |
| `apps/app`, `apps/app-cli`, `apps/app-web` | Native, headless, and browser entrypoints |
| `xtask` | CI, dependency layers, attribution and WASM checks |
| `packaging` | Small configurable packaging scripts |

See [architecture](docs/architecture.md), [commands](docs/commands.md), [MCP](docs/mcp.md), and
[control protocol](docs/control-protocol.md) for contracts and extension points.

## Optional web frontend

```sh
rustup target add wasm32-unknown-unknown
cargo xtask wasm
cargo install trunk --locked
cd apps/app-web
trunk serve
```

Open the address printed by Trunk. Application logic stays in Rust; Trunk generates loading glue.
The web demo uses bounded in-memory storage: Save/Open use the displayed logical path, and
saved state is lost on page reload. Implement browser persistence through the storage boundary
when your application needs it.

## Checks

```sh
cargo xtask ci
```

Runs formatting, clippy with warnings denied, tests, asset attribution, dependency rules, and
WASM validation. CI runs on Linux, macOS, and Windows. Production code forbids unsafe code and
panic-producing shortcuts. Untrusted input and history are bounded.

## Start a project

Use this repository as a template or copy it, rename the `app-*` package prefix and entrypoints,
replace `AppModel`, add commands, then customize the UI and packaging. Follow the exact checklist
in [template usage](docs/template-usage.md). No optional domain crates or product assets are
required to get started.

Licensed under MIT OR Apache-2.0. See `LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE`, and `ATTRIBUTION.md`.
