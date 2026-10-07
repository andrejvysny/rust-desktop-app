# Packaging

Build native binaries first with `cargo build --release -p app -p app-cli`.
Run the script matching the host: `bash packaging/linux/package.sh`,
`bash packaging/macos/package.sh`, or `./packaging/windows/package.ps1`.
These scripts respect `CARGO_TARGET_DIR` and write artifacts to `dist/`.
Linux bundles a desktop entry and binaries; macOS creates an application bundle
and a separate CLI binary; Windows creates a ZIP containing both executables.
The manual GitHub Actions workflow builds these unsigned artifacts without
publishing releases.

Before distributing a product, customize executable names, archive names, the
Linux desktop entry, macOS bundle identifier and versions. Add original or
redistributable application icons with matching `ATTRIBUTION.md` entries.
Replace `example.app-template` with an identifier you own. Configure signing,
notarization, installers and update delivery for your own product as needed;
credentials and release destinations are intentionally absent.

For the optional web frontend, install Trunk (`cargo install trunk --locked`),
install `wasm32-unknown-unknown`, then run `bash packaging/web/package.sh`.
Serve `dist/web/` from a static host. Generated JavaScript is WASM loading glue;
application logic remains Rust. CI validates the Rust WASM build separately.
