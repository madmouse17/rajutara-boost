# Code style and process (distilled from `AGENTS.md` and `CONTRIBUTING.md`)

## Boundaries

- Never open, read, mount or reference anything under the Optitex install folder or any Optitex binary, installer, sample, icon or help file (ADR-0001). Behaviour comes from public documentation; cite the URL in the spec.
- No GPL code ported (ADR-0002). Dual licence `MIT OR Apache-2.0`.
- Use `GLOSSARY.md` words as Rust type and module names; a new domain term goes into `GLOSSARY.md` first.
- No ADR edits; supersede with a new ADR. Never delete `docs/` content.

## Rust rules

- Edition 2024, stable toolchain from `rust-toolchain.toml`. Follow the project skill `.omp/skills/rust-skills`; priorities: ownership, error handling, API design, memory, performance.
- Library crates (`optirust-core`, `-io`, `-nest`, ...): no `unwrap`/`expect` outside tests, no `panic!` on user data, errors as `thiserror` enums, `# Errors` section on every fallible public fn. `optirust-core` has no UI or I/O dependency.
- Geometry in mm as `f64`; never store display units. Tolerances are named constants (`optirust_core::EPS_MM`, `optirust_core::geom`).
- Every public type in `optirust-core` derives `Debug, Clone, PartialEq, serde::{Serialize, Deserialize}`.
- No async runtime. `std::thread` + `std::sync::mpsc`; the UI polls; support cancellation.
- New dependency: justify in one line; licence MIT/Apache/BSD/Zlib/MPL-2.0 (see `deny.toml`); prefer crates already in `Cargo.lock`; `cargo deny check` must pass.
- UI strings through Fluent (`fl!` / `Msg`); English `en` is the source, the `id` `.ftl` updated in the same change.
- Comments explain why, not what. No TODO/FIXME in committed code; open items go to the plan file in `docs/plans/`. Doc comments (`///`) on every public item, in the style of the neighbouring file.
- Importers and exporters: every skipped or approximated entity is an `ImportIssue`.

## Process

- One owner per file per wave. A file you do not own: report `needs-change: <path> — <why>`.
- Never `git commit` or push unless asked. Conventional commits when asked.
- Update `CHANGELOG.md` and the spec or plan the change touches. UI changes come with light and dark screenshots.
- "Done" means the gates in `testing.md` ran and you pasted their result; otherwise say "not verified".
