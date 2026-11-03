<p align="center">
  <img src="assets/rajutara-256.png" alt="rajutara logo: a crab with a woven carapace, holding a needle and thread" width="120">
</p>

<h1 align="center">rajutara-boost</h1>

<p align="center"><b>Guidelines, API reference, runnable examples and an MCP server that let AI coding agents build on <a href="https://github.com/rajutara/rajutara">rajutara</a> correctly.</b></p>

<p align="center">
  <img alt="Licence: MIT or Apache-2.0" src="https://img.shields.io/badge/licence-MIT%20OR%20Apache--2.0-d85820">
  <img alt="Signatures verified against rajutara 1.5.1" src="https://img.shields.io/badge/signatures-verified%20against%201.5.1-111c2e">
</p>

rajutara is a clean-room Rust rewrite of an apparel CAD suite: pattern design, grading, marker nesting and cut planning, as an `egui` app and as headless crates. An agent that has never seen it guesses at the model (millimetres, derived grading, `Stop::Time` vs `Stop::Iterations`) and breaks its rules. rajutara-boost gives the agent the facts, in the shape [Laravel Boost](https://github.com/laravel/boost) does for Laravel: guidelines it loads, an API reference with real signatures, examples that compile, and an MCP server that runs checks.

## What is in it

| Path | What | Agent use |
|---|---|---|
| [`guidelines/rajutara.md`](guidelines/rajutara.md) | What rajutara is, crate map, invariants (mm `f64`, no panic on user data, determinism, clean room). | Always on. |
| [`guidelines/core-model.md`](guidelines/core-model.md) | `Style`, `Piece`, `Contour`, `Notch`, `SizeTable`, `GradeRule`, `Marker` (clusters, cut order, flaws, bump lines, texts), `CutPlan` with real signatures. | Load when touching the model. |
| [`guidelines/io.md`](guidelines/io.md) | Every public loader and writer: `.rjx` `.rjm` `.rjc`, DXF, RUL, HPGL/2, PDF, CSV, Excel `.xlsx`. | Load for file work. |
| [`guidelines/nesting.md`](guidelines/nesting.md) | `nest_marker`, options, `Stop`, determinism, time limits, clusters, flaws, Cutting Order, plotting. | Load for marker work. |
| [`guidelines/grading.md`](guidelines/grading.md), [`cutplan.md`](guidelines/cutplan.md) | Deriving sizes, rule commands, RUL export; Orders, lay solver, reports. | Load per workspace. |
| [`guidelines/ui-modules.md`](guidelines/ui-modules.md) | Adding a workspace `Module`. | Load for UI work. |
| [`guidelines/testing.md`](guidelines/testing.md), [`style.md`](guidelines/style.md) | Gates, the one-binary test layout, mold and nextest, the Optitex-names test, kittest, insta, proptest; the code rules of `AGENTS.md`. | Load before you claim done. |
| [`examples/`](examples/) | `tshirt_sample.rs`, `import_dxf_to_pdf.rs`, `nest_from_rjx.rs`; compiled against rajutara 1.5.1. | Copy and adapt. |
| [`mcp/`](mcp/) | Rust MCP server: `rajutara_smoke`, `rajutara_import_dxf`, `rajutara_nest`, `rajutara_plot`. | Let the agent check its work. |
| [`skills/rajutara/`](skills/rajutara/) | The guidelines packaged as an agent skill. | Skill-aware agents. |
| [`scripts/verify-signatures.py`](scripts/verify-signatures.py) | Checks every signature in the guidelines against a checkout. | Run after an rajutara update. |

## Install

Clone next to your rajutara checkout (the MCP server expects `../rajutara`):

```sh
git clone https://github.com/rajutara/rajutara-boost ~/src/rajutara-boost
```

### Option A: guidelines into `AGENTS.md` / `CLAUDE.md`

Put the always-on file in your project, and point at the rest so the agent loads them on demand:

```sh
cd ~/my-project
mkdir -p .ai && cp -r ~/src/rajutara-boost/guidelines .ai/rajutara
cat >> AGENTS.md <<'MD'

## rajutara
Read `.ai/rajutara/rajutara.md` before any work on rajutara. Then read only the file that matches the task:
core-model.md, io.md, nesting.md, grading.md, cutplan.md, ui-modules.md, testing.md, style.md (same folder).
MD
```

`CLAUDE.md` takes the same block. To inline everything instead: `cat guidelines/rajutara.md >> AGENTS.md`.

### Option B: as a skill

```sh
cp -rL ~/src/rajutara-boost/skills/rajutara ~/.claude/skills/        # Claude Code
cp -rL ~/src/rajutara-boost/skills/rajutara ~/.omp/agent/skills/     # omp
```

`-L` copies the guidelines and examples the skill links to.

### MCP server

```sh
cd ~/src/rajutara-boost/mcp && cargo build --release
claude mcp add rajutara -- ~/src/rajutara-boost/mcp/target/release/rajutara-mcp
```

Codex and omp configuration, the tool list and limits: [`mcp/README.md`](mcp/README.md).

## Examples

```sh
cp examples/*.rs ~/src/rajutara/crates/rajutara/examples/
cd ~/src/rajutara
cargo run --release -p rajutara --example tshirt_sample -- /tmp/tee
```

All three examples and what they print: [`examples/README.md`](examples/README.md).

## Keeping it true

Signatures in the guidelines sit in fenced blocks tagged ```` ```rust sig ````. The pack's version follows the rajutara release it was checked against; [`CHANGELOG.md`](CHANGELOG.md) says what changed. After an rajutara change:

```sh
scripts/verify-signatures.py ~/src/rajutara     # exit 1 and the list of stale lines on drift
```

Verified against rajutara 1.5.1: 438 signature lines, 0 missing.

## Licence

`MIT OR Apache-2.0`, at your option: [`LICENSE-MIT`](LICENSE-MIT), [`LICENSE-APACHE`](LICENSE-APACHE). rajutara is not affiliated with Optitex and uses none of its files or formats.
