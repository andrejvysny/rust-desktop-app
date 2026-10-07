# Commands

Every model change uses `Session::execute(command, params)`. All frontends share these IDs.
Parameters must be JSON objects with exactly the documented fields; unknown fields fail.
`app-cli commands` and the MCP `command_list` tool return metadata and live enablement.

| ID | Pure engine parameters | Behavior |
| --- | --- | --- |
| `app.new` | `{}` | Reset model and history |
| `app.open` | `{"data":{"name":"","counter":0}}` | Validate model, load it, reset history |
| `app.save` | `{}` | Return model object in result `data` |
| `edit.undo` | `{}` | Restore previous model; disabled when history is empty |
| `edit.redo` | `{}` | Restore next model; disabled when redo is empty |
| `demo.incrementCounter` | `{"amount":1}` or `{}` | Checked addition; negative amounts decrement |
| `demo.setName` | `{"name":"Example"}` | Set name, at most 256 UTF-8 bytes |
| `view.togglePanel` | `{}` | Toggle state panel visibility |

`CommandHost`, used by GUI/CLI/MCP/control, also accepts `{"path":"example.json"}` for
`app.open` and `app.save`. Path mode accepts only `path`; mixing `path` and `data` is an error.
Persisted files are plain model JSON, not inspection snapshots or history. The engine's metadata
describes pure data mode; frontend discovery includes storage mode where supported.

Results contain `state`, `events`, and `data`. Inspection state contains `model`, `can_undo`,
`can_redo`, and `view.panel_visible`. Undoable changes emit state/history events. Failed
commands return structured errors without modifying the session. View changes are not undoable.

## CLI examples

```sh
cargo run -p app-cli -- run demo.incrementCounter
cargo run -p app-cli -- run demo.incrementCounter '{"amount":-1}'
cargo run -p app-cli -- run demo.setName '{"name":"Example"}' --save example.json
cargo run -p app-cli -- state --load example.json
cargo run -p app-cli -- run --load example.json \
  --cmd 'demo.incrementCounter={"amount":5}' \
  --cmd 'edit.undo={}' --save example.json
```

Each CLI process starts with a fresh session. Repeated `--cmd ID=JSON` options run in order in
one session. `--load` runs before edits; `--save` runs afterward. Output is JSON, errors go to
stderr, and failures exit nonzero. MCP keeps a session alive across requests.

## Add a command

1. Declare a unique stable ID, description and parameter schema in `crates/engine/src/commands.rs`.
2. Add typed parameter parsing and execution in `crates/engine/src/session.rs`.
3. Route all state mutations through session history helpers where undo is appropriate.
4. Add success, invalid-input and state-preservation tests.
5. Invoke the ID from the UI or another frontend; never reproduce its business logic there.

If you add persistence effects, extend the shared host boundary rather than calling file or UI
APIs from the engine. Maintain the metadata and documentation with the implementation.
