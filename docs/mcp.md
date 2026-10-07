# MCP frontend

The template exposes a newline-delimited JSON-RPC 2.0 MCP server over stdin/stdout. One process
owns one persistent `CommandHost`. Stdout contains protocol responses only; errors go to stderr.

```sh
cargo run -p app-cli -- mcp
```

For an agent client, build the CLI and configure an absolute executable path:

```json
{
  "mcpServers": {
    "app-template": {
      "command": "/absolute/path/to/repository/target/debug/app-cli",
      "args": ["mcp"]
    }
  }
}
```

## Tools

| Tool | Arguments | Result |
| --- | --- | --- |
| `command_list` | `{}` | Command IDs, labels, schemas, undoability, live enablement |
| `command_run` | `{"command":"demo.incrementCounter","params":{}}` | Shared command result |
| `state_get` | `{}` | Model, history availability and presentation state |

First call `initialize`, then send `notifications/initialized`, then use tools. The server
supports protocol revision `2025-06-18`; unsupported requests negotiate this revision.
Tools return text content and structured result content;
execution errors use `isError: true`. Invalid JSON-RPC envelopes use protocol error responses.

## Smoke transcript

Send each object on a separate line, in this order:

```jsonl
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"smoke","version":"1"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/list"}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"command_list","arguments":{}}}
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"command_run","arguments":{"command":"demo.incrementCounter","params":{}}}}
{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"state_get","arguments":{}}}
```

The final model has `counter: 1`. There is no response to the notification. Requests over
1 MiB are rejected without retaining an unbounded frame; the stream recovers at the next line.
Programmatic calls never show dialogs. Native MCP open/save path mode accesses local files with
the permissions of the process; configure the client and working directory accordingly.

This minimal server controls its own headless session. To control a running desktop session,
use the opt-in loopback API in [control protocol](control-protocol.md). A remote MCP bridge is
not part of this template.
