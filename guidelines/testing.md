# Testing

Tests pin behaviour, boundaries and invariants, never incidental formatting. Run from the checkout root.

## Gates (all must pass; paste the results)

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
LIBGL_ALWAYS_SOFTWARE=1 xvfb-run -a -s "-screen 0 1280x800x24" cargo test --workspace
cargo deny check
scripts/test-scripts.sh          # only when you touch scripts/ or the budgets
```

`cargo test --workspace` drives a real window in the UI tests, so it needs a display: `xvfb-run` as above (CI does the same). Scope a quick loop to one crate: `cargo test -p optirust-io`; still run the full gates before you claim done. Windows 7 cross build (`scripts/build-win7.sh`) only when a change touches windowing, dialogs, the file system or dependencies. ADR-0006 budgets are tests: `cargo build --release`, then `scripts/check-budgets.sh size target/release/optirust`.

Disk: parallel builds inflate `target/debug`. Below 20 GB free, delete `target/debug/incremental` before a wave. Never delete `target/*-win7-*`.

## Conventions

- **Where.** Unit tests in `#[cfg(test)] mod tests` beside the code; integration tests in `crates/<crate>/tests/*.rs`. Test files may `#![allow(clippy::unwrap_used, clippy::expect_used)]`; library code may not.
- **Builders.** `optirust_core::testing` (`#[doc(hidden)]`, not a stable API): `polygon`, `contour_of`, `star`, `rect_piece`, `style_with_pieces(&["S","M","L"], base, &[("FRONT", w, h)])`, `rich_piece`, `rich_style`. Reuse them; do not hand-roll a Style in every test.
- **Snapshots (`insta`).** DXF, HPGL, PDF summaries, RUL and JSON shape: `insta::assert_snapshot!` / `assert_json_snapshot!`; files in `tests/snapshots/`. Review changed snapshots; never accept blindly. Output must be deterministic (ids from `IdAlloc`, fixed `created` stamp, `Stop::Iterations`, fixed seed).
- **Properties (`proptest`).** For geometry and grading invariants (`seam_props.rs`, `grading_props.rs`, `validate_props.rs`, DXF round trip). Commit `*.proptest-regressions` files. Use `ProptestConfig::with_cases(128)` unless a case is slow.
- **UI (`egui_kittest`).** Query by accessible label (`h.get_by_label("Grading").click()`), call `h.run()` after each input, assert on state. `optirust-ui/tests/support` has `Fake` modules, `app`, `app_with`, `Scratch` temp dirs. Module crates use `ShellCtx::for_test()` plus the crate's own `i18n::testing::shell()` helper (see `optirust-nest/src/i18n.rs`).
- **i18n.** Loaders are process-global: a test that selects a language flips labels for parallel tests. Use the per-test `testing::loader("en"|"id")`, or serialise with a mutex; skip `i18n_embed::select` when the language is unchanged. Every crate has a parity test (`assert_parity`).
- **Nesting.** Use `Stop::Iterations(n)` and a fixed `seed`, never `Stop::Time`, in assertions. Assert invariants (no overlap via `conflicts`, `unplaced == 0`, efficiency above a floor), not exact coordinates. `nest_env_independent.rs` shows the determinism test shape.
- **Round trips.** Anything written must load back equal: `save_style` then `load_style` equals the original (the T-shirt example checks this).
- **No tests that pin** column widths, float formatting beyond the format's contract, or log text.

## Examples as smoke tests

`cargo build --examples -p optirust` must compile. `cargo run --release -p optirust --example tshirt_sample -- <out_dir>` exercises style, grading, nesting, plot, report and export end to end.
