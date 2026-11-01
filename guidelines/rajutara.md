# rajutara

rajutara is a clean-room Rust rewrite of an apparel CAD suite: pattern design (PDS), grading, marker making (Nesting) and cut planning, in one `egui` desktop window and as headless library crates. Source: <https://github.com/rajutara/rajutara>. Vocabulary is in the checkout's `GLOSSARY.md`; its words are the Rust type and module names. Specs are in `docs/spec/`, settled decisions in `docs/adr/`.

Read this file first, then only the guideline that matches your task:

| Task | File |
|---|---|
| Build or edit a `Style`, `Piece`, `Marker`, `CutPlan` | `core-model.md` |
| Read or write `.optx` `.optm` `.optc` DXF RUL HPGL PDF CSV | `io.md` |
| Nest a marker | `nesting.md` |
| Grade pieces, write RUL | `grading.md` |
| Plan lays from an Order | `cutplan.md` |
| Add a workspace or pane to the app | `ui-modules.md` |
| Write or run tests | `testing.md` |
| Code style, dependencies, process | `style.md` |

## Crates

Dependency direction: `core` <- `io` <- `ui` <- {`pds`, `grading`, `nest`, `cutplan`} <- `rajutara`; `cutplan` also uses `nest`.

| Crate | Owns |
|---|---|
| `rajutara-core` | Domain model, geometry, grading derivation, undo (`Command`, `Document`), units, validation. No UI, no I/O. |
| `rajutara-io` | `.optx`/`.optm`/`.optc` container, ASTM D6673 / AAMA-292 DXF, ASTM RUL, HPGL/2, PDF, CSV, atomic writes. |
| `rajutara-ui` | Shell: `Module` trait, actions, docs, canvas, Fluent i18n, preferences, jobs. |
| `rajutara-pds` | Pattern Design System workspace and the Plot dialog's `plot_job`. |
| `rajutara-grading` | Grading workspace: Size Table and Rule Table commands, grading export. |
| `rajutara-nest` | Marker workspace and the headless Nesting engine (`nest_marker`). |
| `rajutara-cutplan` | Orders, Lay solver, consumption, reports. |
| `rajutara` | The binary: argument parsing and module registration. Examples live in `crates/rajutara/examples/`. |

## Invariants (never break these)

- **Millimetres, `f64`.** Every length in the model is mm, angles are radians counter-clockwise from +X, X right, Y up. Display units are never stored. Tolerances are named constants (`rajutara_core::EPS_MM` = 0.001).
- **No panic on user data.** Library crates return `Result<_, thiserror enum>`; no `unwrap`/`expect`/`panic!` outside tests. Release builds use `panic = "unwind"` (ADR-0009): unwinding is the safety net, not the error path. `nest_marker` catches an engine panic and returns `NestError::Panicked`.
- **Determinism.** Same input and settings give byte-identical output. Ids come from a per-document `IdAlloc`. Nesting takes an explicit `seed`; `Stop::Iterations` is fully reproducible, `Stop::Time` depends on machine speed. The only non-deterministic output is the UTC stamp in export headers (`utc_now_stamp`).
- **Importers never silently drop data.** Everything skipped or approximated is an `ImportIssue`.
- **Clean room (ADR-0001).** Never open, read, decompile or reference any Optitex binary, installer, sample, icon or help file. Optitex behaviour comes from public documentation only; cite the URL in the spec you change. No GPL code (ADR-0002); Seamly2D/Valentina are behaviour references only.
- **No async runtime.** Long work runs on `std::thread` plus `std::sync::mpsc`; the UI polls.
- **Every public type in `rajutara-core`** derives `Debug, Clone, PartialEq, Serialize, Deserialize`.

## Toolchain

Rust stable (`rust-toolchain.toml`: 1.99.1, edition 2024, resolver 3, MSRV 1.95). Gates are in `testing.md`.
