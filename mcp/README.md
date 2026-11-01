# optirust-mcp

A small [MCP](https://modelcontextprotocol.io) server over stdio, built on [`rmcp`](https://crates.io/crates/rmcp) 3.8. It links the optirust library crates directly (no shelling out), so a tool call is a function call: no process start, typed errors, and the same code the app runs.

## Tools

All paths are absolute paths on the machine that runs the server. Errors come back as MCP tool errors (`isError: true`) with the `IoError` / `NestError` / `PlotError` text. Every reply is pretty JSON.

| Tool | Arguments | Returns |
|---|---|---|
| `optirust_smoke` | `path` (`.optx`, `.optm`, `.optc` or `.dxf`) | Loads and validates the file; summary (pieces, sizes, Marker figures, Cut Plan counts) plus validation issues. A cheap "does this file open" check. |
| `optirust_import_dxf` | `path`, `rul?` | Imports an ASTM/AAMA/plain DXF; Style summary (sizes, base size, per-piece vertices, notches, graded points, base-size bounding box in mm), import issues, validation issues. Without `rul` the companion `.rul` named in the DXF is used. |
| `optirust_nest` | `optm`, `seconds`, `out?` | Nests the instances still waiting in the Piece Bar for `seconds` of wall-clock; `efficiency_pct`, `length_mm`, `unplaced`, `overlaps`, validation. Saves the nested Marker to `out` when given; the input file is never changed. A Marker whose instances are all placed returns `nothing to nest`. |
| `optirust_plot` | `optm`, `format` (`pdf` or `hpgl`), `out?` | Plots the placed pieces of a nested Marker: A0 PDF, or HPGL/2 on a roll as wide as the fabric plus 100 mm. Default output: next to the Marker (`.pdf` / `.plt`). Returns the path written. |

`seconds` uses `Stop::Time`, so results depend on machine speed (see `guidelines/nesting.md`).

## Build

The server depends on the optirust crates by path. Clone both repos side by side:

```text
~/src/optirust         # https://github.com/optirust/optirust
~/src/optirust-boost   # this repo
```

```sh
cd ~/src/optirust-boost/mcp
cargo build --release          # first build compiles egui and friends: a few minutes
./check.sh                     # initialize + tools/list over stdio
./check.sh /path/to/style.optx # ... plus one optirust_smoke call
```

If the checkout lives elsewhere, edit the four `path = "../../optirust/crates/..."` lines in `Cargo.toml`. `Cargo.lock` was seeded from the optirust 1.0.0 lock file so dependency versions match the app. Use the same Rust toolchain as the checkout (`rust-toolchain.toml`, 1.99.1).

The binary is `mcp/target/release/optirust-mcp`.

## Setup

Replace `/abs/path` with the real path of the binary.

### Claude Code

```sh
claude mcp add optirust -- /abs/path/optirust-boost/mcp/target/release/optirust-mcp
claude mcp list
```

Or a project `.mcp.json`:

```json
{ "mcpServers": { "optirust": { "command": "/abs/path/optirust-boost/mcp/target/release/optirust-mcp" } } }
```

### Codex

`~/.codex/config.toml`:

```toml
[mcp_servers.optirust]
command = "/abs/path/optirust-boost/mcp/target/release/optirust-mcp"
```

### omp (oh-my-pi)

`~/.omp/agent/mcp.json` (or the project's `.omp/mcp.json`):

```json
{
  "mcpServers": {
    "optirust": {
      "type": "stdio",
      "command": "/abs/path/optirust-boost/mcp/target/release/optirust-mcp",
      "args": []
    }
  }
}
```

## Protocol check by hand

```sh
printf '%s\n' \
 '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"x","version":"0"}}}' \
 '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
 '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | ./target/release/optirust-mcp
```

The server exits when stdin closes; `check.sh` keeps stdin open for 2 s so replies are flushed.

## Limits

- One process, one request at a time per tool call; a long `optirust_nest` blocks that call only.
- No resources or prompts; tools only.
- Paths are not sandboxed: the server reads and writes what the user running it can. Run it as that user, not as root.
