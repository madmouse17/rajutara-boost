# Changelog

All notable changes to rajutara-boost. Format: [Keep a Changelog](https://keepachangelog.com/). The version follows the rajutara release the guidelines, examples and MCP server were checked against.

## [1.5.0] - 2026-11-04

Checked against rajutara 1.5.0: `scripts/verify-signatures.py` passes and the MCP server builds against the checkout.

### Added
- `guidelines/core-model.md`: the 1.5.0 model with real signatures: Marker `clusters`, `cut_order`, `bump_lines`, `cut_lines`, `flaws`, `texts` and their types; Style `equal_segments`, `fabric`; Piece `stack`, `placement`, `spread`, `zones`, `from_zone`, `color`; `Vertex::name`; `StackPoint::AlongLine`; label `one_line` and `separator`.
- `guidelines/io.md`: Excel workbooks (`write_xlsx`, `rajutara_io::xlsx::{Sheet, write_workbook, render_workbook}`), `import_dxf_with_rul_cancellable`, `DxfFileInfo` and `DxfImportOptions::line_mode`.
- `guidelines/nesting.md`: what the engine respects (Fix on Marker, clusters and Fuse Blocks, flaws, Insert to Hole) and the Cutting Order behind the cut file.
- Wave B signatures: `guidelines/nesting.md` pins Bump/ Cut Lines (`set_bump_lines`, `add_bump_line`, `move_bump_line`, `marker::bump`), Marker texts (`add_text` and friends, `TextSpec`), `FlawSpec` and `flaws::parse_csv`, Substitute Size and the Excel report (`report::sheets`, `write_xlsx_report`); `guidelines/core-model.md` pins the PDS Equal Segments and Zones operations (`equal_segments::enforce`, `zones::add_zone`, `create_pieces_from_map`, `correct_pieces_by_map`).
- `guidelines/testing.md`: the one-binary-per-crate test layout (`tests/all/`), `debug = "line-tables-only"`, mold and cargo-nextest, and the Optitex-names test.
- `rajutara_smoke` reports pinned pieces, clusters, cut order steps and flaws of a Marker.
- This changelog.

### Changed
- Native files are `.rjx`, `.rjm`, `.rjc` everywhere. The MCP server opens them by those extensions (and still the pre-rename `.optx`, `.optm`, `.optc`); before, it refused the new ones.
- MCP: the Marker argument of `rajutara_nest` and `rajutara_plot` is now `marker` (was `optm`). The crate version is 1.5.0 (was 0.1.0).
- `examples/nest_from_optx.rs` is now `examples/nest_from_rjx.rs`; `examples/tshirt_sample.rs` matches the copy in rajutara (its marker report is now `.xlsx`). The Marker report writer is `write_xlsx_report`; the CSV report writer of 1.4.0 is gone.
- `guidelines/ui-modules.md` names the `Module::dialogs` hook and the Optitex-names step; `style.md` drops paths rajutara no longer has (`docs/plans/`, `.omp/skills/`).

## [1.0.0] - 2026-11-01

First release: guidelines, examples and MCP server for rajutara 1.0.0.
