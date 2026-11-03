# Examples

Three headless programs written against the real rajutara APIs. They are not part of this repo's build: they compile inside an rajutara checkout.

| File | What it does | Run |
|---|---|---|
| `tshirt_sample.rs` | Drafts a short-sleeve T-shirt Style, grades S/M/L/XL, nests a marker on 1800 mm jersey, writes `.rjx`, `.rjm`, PDF, HPGL, Excel `.xlsx` report, DXF + RUL and reloads everything. | `... --example tshirt_sample -- <out_dir>` |
| `import_dxf_to_pdf.rs` | Imports an ASTM/AAMA/plain DXF with its RUL grade rules, plots every size of every piece to one A0 PDF. | `... --example import_dxf_to_pdf -- <in.dxf> <out.pdf> [rules.rul]` |
| `nest_from_rjx.rs` | Loads a `.rjx`, builds a marker with one Bundle per size, nests it, saves `.rjm` and plots HPGL/2. | `... --example nest_from_rjx -- <style.rjx> <out_stem> [seconds=10] [width_mm=1800]` |

`...` is `cargo run --release -p rajutara` from the rajutara checkout root.

## Run against a checkout

```sh
git clone https://github.com/rajutara/rajutara ~/src/rajutara      # or use your existing checkout
cp examples/*.rs ~/src/rajutara/crates/rajutara/examples/
cd ~/src/rajutara
cargo run --release -p rajutara --example tshirt_sample -- /tmp/tee
cargo run --release -p rajutara --example import_dxf_to_pdf -- /tmp/tee/kaos-lengan-pendek.dxf /tmp/tee/all-sizes.pdf
cargo run --release -p rajutara --example nest_from_rjx -- /tmp/tee/kaos-lengan-pendek.rjx /tmp/tee/nested 10 1800
```

`tshirt_sample.rs` is already in the main repo; copying it again is harmless. The ready-made sample files are in the checkout's `samples/kaos-lengan-pendek/`. Delete the copies from the checkout when you are done (do not commit them there).

Why these crates: the examples use `rajutara-core`, `-io`, `-nest`, `-grading`, `-pds` and `-ui`, all already dependencies of the `rajutara` package, so no `Cargo.toml` change is needed.

## Compiled proof

Built on 2026-11-04 against the rajutara 1.5.1 checkout (rustc 1.99.1, edition 2024), from a scratch crate with path dependencies on the six crates above (the same code paths as copying the files into `crates/rajutara/examples/`):

```text
$ cargo build --release --examples
    Finished `release` profile [optimized] target(s) in 37.99s
```

Executed once, writing into an empty folder:

```text
tshirt_sample:     marker: 20 pieces placed of 20 (4 bundles), efficiency 84.49 %, length 2.519 m, 0 conflicts
                   reload: kaos-lengan-pendek.rjx (4 pieces, 0 issues) and kaos-lengan-pendek-marker.rjm (20 instances, 0 issues) load equal
import_dxf_to_pdf: style "Kaos Lengan Pendek": 4 pieces, sizes ["S", "M", "L", "XL"] (base M), 20 grade rules, 3 import issues, 0 validation errors
nest_from_rjx:     nested: efficiency 84.28 %, length 2.525 m, 0 unplaced, 0 validation issue(s); wrote nested.rjm and nested.plt
```

Efficiency and length come from a 15 s (`tshirt_sample`) or 10 s (`nest_from_rjx`) `Stop::Time` run, so they vary with machine speed.
