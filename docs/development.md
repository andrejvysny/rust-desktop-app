# Development

Use Rust 1.95 or newer with Cargo, rustfmt and clippy. Install the WASM target before running CI:

```sh
rustup component add rustfmt clippy
rustup target add wasm32-unknown-unknown
```

On Linux install native build prerequisites, for example on Debian/Ubuntu:

```sh
sudo apt-get install build-essential pkg-config libxkbcommon-dev libwayland-dev \
  libx11-dev libxrandr-dev libxi-dev libgl1-mesa-dev
```

macOS needs Xcode Command Line Tools; Windows needs the MSVC Rust toolchain and C++ build tools.
The desktop UI needs a usable display/graphics backend. Engine, CLI and MCP tests need no window.
Dev/test debug info is disabled to keep template builds smaller; enable it locally when debugging.

## Quality gates

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo xtask layers
cargo xtask assets
cargo xtask wasm
cargo xtask ci
```

`ci` runs all gates and stops on the first failure. `wasm` checks portable library packages and
builds `app-web` for `wasm32-unknown-unknown`; missing targets fail with an actionable error.
This proves compilation, not browser runtime behavior. GitHub CI runs the full suite on the
three primary desktop platforms, plus an MSRV check.

Tests cover command execution and hostile input, model serialization, bounded/branching undo,
failed persistence, headless CLI and MCP lifecycle/framing. UI acceptance tests exercise actual
egui controls without a native window. Use `app-testkit` as a dev-dependency for shared command
fixtures; do not add it to production dependencies.

## Frontend smoke checks

```sh
cargo run -p app
cargo run -p app-cli -- commands
cargo run -p app-cli -- run demo.incrementCounter
```

Look at the native window. Change the name, increment/decrement, undo/redo, toggle the state
panel, save, reset, and reopen. Verify state tracks commands. Native open/save dialogs are
injected by `apps/app`; the shared UI and engine do not import rfd.

Run the transcript in [MCP](mcp.md) through `app-cli mcp`; confirm command discovery and resulting
counter state. For the running window, use the [loopback control API](control-protocol.md).
There is no default server or network connection.

## Web

```sh
cargo install trunk --locked
cd apps/app-web
trunk serve
```

The same Rust UI runs through eframe's web runner. Trunk supplies generated loading glue; no
handwritten JavaScript/TypeScript application logic is needed. Open the printed address and
repeat counter/name/undo checks. Web persistence is temporary in-memory storage and is cleared
on reload. Production browser storage requires a frontend implementation of `Storage`.

## Packaging

See [packaging](../packaging/README.md) for native bundles/archives and optional Trunk packaging.
The included workflow creates build artifacts, not published or signed releases. Set product
names, identifiers, icons, signing and update policy for each project before distributing it.
Cross-platform runtime checks remain separate from local compilation.
