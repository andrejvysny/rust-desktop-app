# Desktop control protocol

The desktop app can expose an opt-in JSON-lines TCP API for local automation:

```sh
cargo run -p app -- --control 7981
```

It binds only `127.0.0.1`; requests run on the UI thread through the same `CommandHost` as
buttons and menus. Each line is one request and receives one JSON response. This control
protocol is separate from MCP and does not require its initialization handshake.

```jsonl
{"id":1,"method":"engine.commands"}
{"id":2,"method":"engine.execute","params":{"command":"demo.incrementCounter","params":{}}}
{"id":3,"method":"state.get"}
```

- `engine.commands`: registry metadata with current enablement.
- `engine.execute`: `params.command` is a stable command ID; `params.params` is its JSON object.
- `state.get`: inspect model, history availability and presentation state.

Responses preserve request IDs and contain either `result` or `error`. Calls never open file
dialogs; use `app.open`/`app.save` with `path` explicitly. Invalid JSON and unknown methods return
errors. Inputs are limited to 1 MiB; idle clients expire after 30 seconds and connections are
limited to 16. No remote listening interface is provided.

The API has no authentication. Any local process able to connect can execute commands,
including native file open/save. Enable it only when needed, and do not forward its port to
untrusted clients. The default application starts without a listener.
