<p align="center">
  <img src="assets/optirust-256.png" alt="optirust logo: a crab with a woven carapace, holding a needle and thread" width="120">
</p>

<h1 align="center">optirust-boost</h1>

<p align="center"><b>Guidelines, API reference, runnable examples and an MCP server that let AI coding agents build on <a href="https://github.com/optirust/optirust">optirust</a> correctly.</b></p>

<p align="center">
  <img alt="Licence: MIT or Apache-2.0" src="https://img.shields.io/badge/licence-MIT%20OR%20Apache--2.0-d85820">
  <img alt="Signatures verified against optirust 1.0.0" src="https://img.shields.io/badge/signatures-verified%20against%201.0.0-111c2e">
</p>

optirust is a clean-room Rust rewrite of an apparel CAD suite: pattern design, grading, marker nesting and cut planning, as an `egui` app and as headless crates. An agent that has never seen it guesses at the model (millimetres, derived grading, `Stop::Time` vs `Stop::Iterations`) and breaks its rules. optirust-boost gives the agent the facts, in the shape [Laravel Boost](https://github.com/laravel/boost) does for Laravel: guidelines it loads, an API reference with real signatures, examples that compile, and an MCP server that runs checks.

## What is in it

| Path | What | Agent use |
|---|---|---|
| [`guidelines/optirust.md`](guidelines/optirust.md) | What optirust is, crate map, invariants (mm `f64`, no panic on user data, determinism, clean room). | Always on. |
| [`guidelines/core-model.md`](guidelines/core-model.md) | `Style`, `Piece`, `Contour`, `Notch`, `SizeTable`, `GradeRule`, `Marker`, `CutPlan` with real signatures. | Load when touching the model. |
| [`guidelines/io.md`](guidelines/io.md) | Every public loader and writer: `.optx` `.optm` `.optc`, DXF, RUL, HPGL/2, PDF, CSV. | Load for file work. |
| [`guidelines/nesting.md`](guidelines/nesting.md) | `nest_marker`, options, `Stop`, determinism, time limits, plotting. | Load for marker work. |
| [`guidelines/grading.md`](guidelines/grading.md), [`cutplan.md`](guidelines/cutplan.md) | Deriving sizes, rule commands, RUL export; Orders, lay solver, reports. | Load per workspace. |
| [`guidelines/ui-modules.md`](guidelines/ui-modules.md) | Adding a workspace `Module`. | Load for UI work. |
| [`guidelines/testing.md`](guidelines/testing.md), [`style.md`](guidelines/style.md) | Gates, kittest, insta, proptest; the code rules of `AGENTS.md`. | Load before you claim done. |
| [`examples/`](examples/) | `tshirt_sample.rs`, `import_dxf_to_pdf.rs`, `nest_from_optx.rs`; compiled against optirust 1.0.0. | Copy and adapt. |
| [`mcp/`](mcp/) | Rust MCP server: `optirust_smoke`, `optirust_import_dxf`, `optirust_nest`, `optirust_plot`. | Let the agent check its work. |
| [`skills/optirust/`](skills/optirust/) | The guidelines packaged as an agent skill. | Skill-aware agents. |
| [`scripts/verify-signatures.py`](scripts/verify-signatures.py) | Checks every signature in the guidelines against a checkout. | Run after an optirust update. |

## Install

Clone next to your optirust checkout (the MCP server expects `../optirust`):

```sh
git clone https://github.com/optirust/optirust-boost ~/src/optirust-boost
```

### Option A: guidelines into `AGENTS.md` / `CLAUDE.md`

Put the always-on file in your project, and point at the rest so the agent loads them on demand:

```sh
cd ~/my-project
mkdir -p .ai && cp -r ~/src/optirust-boost/guidelines .ai/optirust
cat >> AGENTS.md <<'MD'

## optirust
Read `.ai/optirust/optirust.md` before any work on optirust. Then read only the file that matches the task:
core-model.md, io.md, nesting.md, grading.md, cutplan.md, ui-modules.md, testing.md, style.md (same folder).
MD
```

`CLAUDE.md` takes the same block. To inline everything instead: `cat guidelines/optirust.md >> AGENTS.md`.

### Option B: as a skill

```sh
cp -rL ~/src/optirust-boost/skills/optirust ~/.claude/skills/        # Claude Code
cp -rL ~/src/optirust-boost/skills/optirust ~/.omp/agent/skills/     # omp
```

`-L` copies the guidelines and examples the skill links to.

### MCP server

```sh
cd ~/src/optirust-boost/mcp && cargo build --release
claude mcp add optirust -- ~/src/optirust-boost/mcp/target/release/optirust-mcp
```

Codex and omp configuration, the tool list and limits: [`mcp/README.md`](mcp/README.md).

## Examples

```sh
cp examples/*.rs ~/src/optirust/crates/optirust/examples/
cd ~/src/optirust
cargo run --release -p optirust --example tshirt_sample -- /tmp/tee
```

All three examples and what they print: [`examples/README.md`](examples/README.md).

## Keeping it true

Signatures in the guidelines sit in fenced blocks tagged ```` ```rust sig ````. After an optirust change:

```sh
scripts/verify-signatures.py ~/src/optirust     # exit 1 and the list of stale lines on drift
```

Verified against optirust 1.0.0: 307 signature lines, 0 missing.

## Licence

`MIT OR Apache-2.0`, at your option: [`LICENSE-MIT`](LICENSE-MIT), [`LICENSE-APACHE`](LICENSE-APACHE). optirust is not affiliated with Optitex and uses none of its files or formats.
