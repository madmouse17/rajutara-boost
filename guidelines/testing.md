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

`cargo test --workspace` drives a real window in the UI tests, so it needs a display: `xvfb-run` as above (CI does the same). Scope a quick loop to one crate: `cargo test -p rajutara-io`; still run the full gates before you claim done. Windows builds are cross-built and checked under Wine by the maintainer's tooling outside the repository; say in the pull request when a change touches windowing, dialogs, the file system or dependencies. ADR-0006 budgets are tests: `cargo build --release`, then `scripts/check-budgets.sh size target/release/rajutara`.

Disk: parallel builds inflate `target/debug`. Below 20 GB free, delete `target/debug/incremental` before a wave.

## Layout: one integration binary per crate

Each crate merges its integration tests into one binary, so a test build links once per crate instead of once per file:

```text
crates/<crate>/tests/all/main.rs        //! ... merged into one binary so the test build links once.
                                        mod <file>;   one line per test file
crates/<crate>/tests/all/<file>.rs      the tests
crates/<crate>/tests/all/support/       shared helpers (common/ in core, io, cutplan, grading)
crates/<crate>/tests/all/snapshots/     insta files, named all__<file>__<name>.snap
```

- A new test file needs its `mod <file>;` line in `tests/all/main.rs`; without it the file is never compiled and its tests silently never run.
- Do not add a new top-level `tests/<file>.rs`: each one is another binary to link. The one exception is `crates/rajutara-nest/tests/nest_env_independent.rs`, which changes the process environment and so runs alone.
- Select tests by module path: `cargo test -p rajutara-pds --test all notch_slit`, `cargo test -p rajutara --test all e2e_workflow -- --nocapture`. Release-only benchmarks: `cargo test -p rajutara --release --test all e2e_workflow -- --include-ignored`.

## Faster local builds

- `Cargo.toml` sets `[profile.dev] debug = "line-tables-only"`: backtraces keep file and line, debug info is a fraction of the size, links are faster. For a debugger with locals: `CARGO_PROFILE_DEV_DEBUG=true cargo test -p <crate>`.
- `mold` as the linker is a per-machine choice, never committed: in `~/.cargo/config.toml`, `[target.x86_64-unknown-linux-gnu]` with `rustflags = ["-C", "link-arg=-fuse-ld=mold"]`. A link error naming `ld.mold` means `mold` is not on `PATH`.
- `cargo nextest run -p <crate>` (cargo-nextest) is optional; it runs each test in its own process. CI runs `cargo test`, so a test must pass there too: do not rely on nextest's process isolation to hide shared global state (see i18n below).

## Optitex names

`crates/rajutara/tests/all/optitex_names.rs` reads the Fluent files of `pds`, `nest`, `grading`, `ui` and `cutplan`:

- every action id (`rajutara_pds::actions::ALL_IDS`, the `nest-act-*` and `grading-*` ids of `ui/actions.rs`) has a message with a `.optitex` attribute holding the public Optitex name;
- the English label equals that name (a trailing `…` for a dialog and ` window` for a pane are ignored), unless the id is in `EXCEPTIONS` with a one-line reason; a stale exception fails;
- the `id` file repeats the English `.optitex` value unchanged;
- every `.menu` caption differs from its label, has a distinct Indonesian caption and is pinned in `MENU_CAPTIONS`;
- every `*-prompt-*` message is a sentence (capital first, full stop last) in both languages.

```ftl
nest-act-cluster = Cluster Pieces
    .optitex = Cluster Pieces
```

`crates/rajutara-pds/tests/all/ids_in_catalog.rs` treats every `pds-` and `cmd-` string literal in `src/` as a message id that must exist in both languages, so never start an egui id salt with `pds-`. `crates/rajutara-pds/tests/all/optitex_keys.rs` pins the PDS keys of the public Optitex list and refuses a key bound twice.

## Conventions

- **Where.** Unit tests in `#[cfg(test)] mod tests` beside the code; integration tests in `crates/<crate>/tests/all/`. Test files may `#![allow(clippy::unwrap_used, clippy::expect_used)]`; library code may not.
- **Builders.** `rajutara_core::testing` (`#[doc(hidden)]`, not a stable API): `polygon`, `contour_of`, `star`, `rect_piece`, `style_with_pieces(&["S","M","L"], base, &[("FRONT", w, h)])`, `rich_piece`, `rich_style`. Reuse them; do not hand-roll a Style in every test.
- **Snapshots (`insta`).** DXF, HPGL, PDF summaries, RUL and JSON shape: `insta::assert_snapshot!` / `assert_json_snapshot!`; files in `tests/all/snapshots/`. Review changed snapshots; never accept blindly. Output must be deterministic (ids from `IdAlloc`, fixed `created` stamp, `Stop::Iterations`, fixed seed).
- **Properties (`proptest`).** For geometry and grading invariants (`seam_props.rs`, `grading_props.rs`, `validate_props.rs`, DXF round trip). Commit `*.proptest-regressions` files. Use `ProptestConfig::with_cases(128)` unless a case is slow.
- **UI (`egui_kittest`).** Query by accessible label (`h.get_by_label("Grading").click()`), call `h.run()` after each input, assert on state. `crates/rajutara-ui/tests/all/support/` has `Fake` modules, `app`, `app_with`, `Scratch` temp dirs. Module crates use `ShellCtx::for_test()` plus the crate's own `i18n::testing::shell()` helper (see `rajutara-nest/src/i18n.rs`). The PDS pane harness (`crates/rajutara-pds/tests/all/support/harness.rs`) has no module actions or catalog registered: register them before testing menus, and query menu buttons with `label_contains` (labels include the shortcut).
- **i18n.** Loaders are process-global: a test that selects a language flips labels for parallel tests. Use the per-test `testing::loader("en"|"id")`, or serialise with a mutex; skip `i18n_embed::select` when the language is unchanged. Every crate has a parity test (`assert_parity`).
- **Nesting.** Use `Stop::Iterations(n)` and a fixed `seed`, never `Stop::Time`, in assertions. Assert invariants (no overlap via `conflicts`, `unplaced == 0`, efficiency above a floor), not exact coordinates. `nest_env_independent.rs` shows the determinism test shape.
- **Round trips.** Anything written must load back equal: `save_style` then `load_style` equals the original (the T-shirt example checks this). A new optional model field also needs a test that a document without it writes no new key.
- **No tests that pin** column widths, float formatting beyond the format's contract, or log text.

## Examples as smoke tests

`cargo build --examples -p rajutara` must compile. `cargo run --release -p rajutara --example tshirt_sample -- <out_dir>` exercises style, grading, nesting, plot, report and export end to end.
