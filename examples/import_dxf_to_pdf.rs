//! Imports an ASTM / AAMA / plain DXF (with its ASTM RUL grade rules) and plots every size of every piece to
//! one PDF, nested flat on an A0-wide sheet.
//!
//! `cargo run --release -p rajutara --example import_dxf_to_pdf -- <in.dxf> <out.pdf> [rules.rul]`
//!
//! Without `rules.rul` the importer reads the `.rul` file the DXF names in `Grade Rule Table:` from the
//! DXF's own folder. A `.rul` given on the command line wins.

use std::error::Error;
use std::fs;
use std::path::Path;

use rajutara_core::{Severity, validate_style};
use rajutara_io::{DxfImportOptions, PdfPage, import_dxf, parse_dxf, plot_pdf};
use rajutara_pds::plot::job::plot_job;
use rajutara_pds::tools::{FieldValue, Values};
use rajutara_ui::Selection;

type Res<T> = Result<T, Box<dyn Error>>;

fn main() -> Res<()> {
    let mut args = std::env::args().skip(1);
    let (Some(dxf), Some(out)) = (args.next(), args.next()) else {
        return Err("usage: import_dxf_to_pdf <in.dxf> <out.pdf> [rules.rul]".into());
    };
    let rul = args.next();
    let opts = DxfImportOptions::default();

    let (style, issues) = match rul {
        Some(rul) => parse_dxf(&fs::read(&dxf)?, Some(&fs::read_to_string(rul)?), &opts)?,
        None => import_dxf(Path::new(&dxf), &opts)?,
    };
    for issue in &issues {
        eprintln!(
            "import {:?} {}: {}",
            issue.severity, issue.location, issue.message
        );
    }

    let errors = validate_style(&style)
        .into_iter()
        .filter(|i| i.severity == Severity::Error)
        .count();
    println!(
        "style {:?}: {} pieces, sizes {:?} (base {}), {} grade rules, {} import issues, {} validation errors",
        style.name,
        style.pieces.len(),
        style
            .size_table
            .sizes
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<_>>(),
        style.size_table.sizes[style.size_table.base].name,
        style.rule_table.rules.len(),
        issues.len(),
        errors
    );

    // "sizes" choice 1 = every size of the Size Table; the other keys keep the Plot dialog defaults.
    let values = Values(vec![
        ("sizes", FieldValue::Choice(1)),
        ("target", FieldValue::Choice(1)),
    ]);
    let (job, _) = plot_job(&style, &Selection::default(), &values)?;
    let warnings = plot_pdf(&job, PdfPage::A0, Path::new(&out))?;
    println!("wrote {out} ({} writer warnings)", warnings.len());
    Ok(())
}
