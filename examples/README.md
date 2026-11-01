# Examples

Three headless programs written against the real optirust APIs. They are not part of this repo's build: they compile inside an optirust checkout.

| File | What it does | Run |
|---|---|---|
| `tshirt_sample.rs` | Drafts a short-sleeve T-shirt Style, grades S/M/L/XL, nests a marker on 1800 mm jersey, writes `.optx`, `.optm`, PDF, HPGL, CSV report, DXF + RUL and reloads everything. | `... --example tshirt_sample -- <out_dir>` |
| `import_dxf_to_pdf.rs` | Imports an ASTM/AAMA/plain DXF with its RUL grade rules, plots every size of every piece to one A0 PDF. | `... --example import_dxf_to_pdf -- <in.dxf> <out.pdf> [rules.rul]` |
| `nest_from_optx.rs` | Loads a `.optx`, builds a marker with one Bundle per size, nests it, saves `.optm` and plots HPGL/2. | `... --example nest_from_optx -- <style.optx> <out_stem> [seconds=10] [width_mm=1800]` |

`...` is `cargo run --release -p optirust` from the optirust checkout root.

## Run against a checkout

```sh
git clone https://github.com/optirust/optirust ~/src/optirust      # or use your existing checkout
cp examples/*.rs ~/src/optirust/crates/optirust/examples/
cd ~/src/optirust
cargo run --release -p optirust --example tshirt_sample -- /tmp/tee
cargo run --release -p optirust --example import_dxf_to_pdf -- /tmp/tee/kaos-lengan-pendek.dxf /tmp/tee/all-sizes.pdf
cargo run --release -p optirust --example nest_from_optx -- /tmp/tee/kaos-lengan-pendek.optx /tmp/tee/nested 10 1800
```

`tshirt_sample.rs` is already in the main repo; copying it again is harmless. The ready-made sample files are in the checkout's `samples/kaos-lengan-pendek/`. Delete the copies from the checkout when you are done (do not commit them there).

Why these crates: the examples use `optirust-core`, `-io`, `-nest`, `-grading`, `-pds` and `-ui`, all already dependencies of the `optirust` package, so no `Cargo.toml` change is needed.

## Compiled proof

Dropped into `crates/optirust/examples/` of an optirust 1.0.0 checkout on 2026-11-01 (rustc 1.99.1, edition 2024) and built:

```text
$ cargo build --examples -p optirust
   Compiling optirust v1.0.0 (.../crates/optirust)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.16s
```

Executed once against `samples/kaos-lengan-pendek/`:

```text
import_dxf_to_pdf: style "Kaos Lengan Pendek": 4 pieces, sizes ["S","M","L","XL"] (base M), 20 grade rules, 3 import issues, 0 validation errors
nest_from_optx:    nested: efficiency 84.23 %, length 2.527 m, 0 unplaced, 0 validation issue(s); wrote .optm and .plt
```
