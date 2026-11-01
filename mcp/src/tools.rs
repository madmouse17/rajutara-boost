//! The four tool bodies: plain functions over the rajutara library crates, no MCP types.

use std::path::{Path, PathBuf};
use std::time::Duration;

use rajutara_core::Unit;
use rajutara_core::{Issue, Marker, Severity, Style, grading::piece_for_size, validate_marker};
use rajutara_io::PdfPage;
use rajutara_io::{
    DxfImportOptions, ImportIssue, IoError, PlotterSetup, import_dxf, load_cut_plan, load_marker,
    load_style, parse_dxf, save_marker, utc_now_stamp,
};
use rajutara_nest::engine::{Control, Stop};
use rajutara_nest::marker::instances::ShapeBook;
use rajutara_nest::marker::metrics::{conflicts, metrics};
use rajutara_nest::output::plot::{
    PlotTarget, header_lines, marker_job, plot_marker, setup_from_json,
};
use rajutara_nest::report::ReportFmt;
use rajutara_nest::{NestMarkerOptions, nest_marker};
use rajutara_ui::{I18n, Lang};
use serde_json::{Value, json};

/// Findings listed in a reply; the rest is counted.
const MAX_LISTED: usize = 25;

pub type ToolResult = Result<Value, String>;

fn io_err(e: IoError) -> String {
    e.to_string()
}

fn listed<T: ToString>(items: impl IntoIterator<Item = T>) -> Value {
    let all: Vec<String> = items.into_iter().map(|i| i.to_string()).collect();
    json!({ "count": all.len(), "first": all.iter().take(MAX_LISTED).collect::<Vec<_>>() })
}

fn issues(found: &[Issue]) -> Value {
    let errors = found
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .count();
    json!({
        "errors": errors,
        "warnings": found.len() - errors,
        "list": listed(found.iter().map(|i| format!("{:?} {:?}", i.severity, i.code))),
    })
}

fn import_issues(found: &[ImportIssue]) -> Value {
    listed(
        found
            .iter()
            .map(|i| format!("{:?} {}: {}", i.severity, i.location, i.message)),
    )
}

fn style_summary(style: &Style) -> Value {
    let table = &style.size_table;
    let pieces: Vec<Value> = style
        .pieces
        .iter()
        .map(|p| {
            let base = piece_for_size(p, table, table.base).contour.bounding_box();
            json!({
                "name": p.name,
                "quantity": p.quantity,
                "pair": p.pair,
                "material": p.material,
                "vertices": p.contour.points.len(),
                "notches": p.notches.len(),
                "graded_points": p.grading.len(),
                "base_size_bbox_mm": [round2(base.width()), round2(base.height())],
            })
        })
        .collect();
    json!({
        "name": style.name,
        "sizes": table.sizes.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
        "base_size": table.sizes.get(table.base).map(|s| s.name.as_str()),
        "grade_rules": style.rule_table.rules.len(),
        "pieces": pieces,
    })
}

fn marker_summary(marker: &Marker) -> Value {
    let mut book = ShapeBook::default();
    let figures = metrics(marker, &mut book);
    json!({
        "name": marker.name,
        "material": marker.material.name,
        "width_mm": marker.material.width,
        "styles": marker.styles.len(),
        "instances": figures.requested,
        "placed": figures.placed,
        "used_length_mm": round2(figures.used_length),
        "efficiency_pct": round2(figures.efficiency_pct),
    })
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

/// Loads `path` by extension (`.optx`, `.optm`, `.optc`, `.dxf`), validates it and summarises it.
pub fn smoke(path: &Path) -> ToolResult {
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "optx" => {
            let (style, found) = load_style(path).map_err(io_err)?;
            Ok(
                json!({ "kind": "style", "summary": style_summary(&style), "issues": issues(&found) }),
            )
        }
        "optm" => {
            let (marker, found) = load_marker(path).map_err(io_err)?;
            Ok(
                json!({ "kind": "marker", "summary": marker_summary(&marker), "issues": issues(&found) }),
            )
        }
        "optc" => {
            let (plan, found) = load_cut_plan(path).map_err(io_err)?;
            Ok(json!({
                "kind": "cut plan",
                "name": plan.name,
                "style": plan.style.style.name,
                "order_lines": plan.order.lines.len(),
                "lays": plan.lays.len(),
                "markers": plan.markers.len(),
                "issues": issues(&found),
            }))
        }
        "dxf" => import(path, None),
        _ => Err(format!(
            "{}: expected .optx, .optm, .optc or .dxf",
            path.display()
        )),
    }
}

/// Imports a DXF (and the `rul` file, else the companion `.rul` the DXF names) and summarises the Style.
pub fn import(dxf: &Path, rul: Option<&Path>) -> ToolResult {
    let opts = DxfImportOptions::default();
    let (style, found) = match rul {
        Some(rul) => {
            let bytes = std::fs::read(dxf).map_err(|e| format!("{}: {e}", dxf.display()))?;
            let text =
                std::fs::read_to_string(rul).map_err(|e| format!("{}: {e}", rul.display()))?;
            parse_dxf(&bytes, Some(&text), &opts)
        }
        None => import_dxf(dxf, &opts),
    }
    .map_err(io_err)?;
    Ok(json!({
        "kind": "dxf import",
        "summary": style_summary(&style),
        "validation": issues(&rajutara_core::validate_style(&style)),
        "import_issues": import_issues(&found),
    }))
}

/// Auto-nests every waiting instance of the marker for `seconds`; saves the result to `out` when given.
pub fn nest(optm: &Path, seconds: u64, out: Option<&Path>) -> ToolResult {
    if seconds == 0 {
        return Err("seconds must be at least 1".to_owned());
    }
    let (marker, _) = load_marker(optm).map_err(io_err)?;
    let options = NestMarkerOptions {
        stop: Stop::Time(Duration::from_secs(seconds)),
        ..NestMarkerOptions::default()
    };
    let nested =
        nest_marker(&marker, &options, &|_| Control::Continue).map_err(|e| e.to_string())?;
    let mut book = ShapeBook::default();
    let found = conflicts(&nested.marker, &mut book);
    let invalid = validate_marker(&nested.marker);
    if let Some(out) = out {
        save_marker(&nested.marker, out).map_err(io_err)?;
    }
    Ok(json!({
        "efficiency_pct": round2(nested.efficiency_pct),
        "length_mm": round2(nested.length),
        "unplaced": nested.unplaced,
        "overlaps": found.overlap_count(),
        "validation": issues(&invalid),
        "saved_to": out.map(|p| p.display().to_string()),
    }))
}

/// Plots the placed pieces of the marker as `pdf` (A0) or `hpgl` (roll as wide as the fabric plus 100 mm).
pub fn plot(optm: &Path, format: &str, out: Option<&Path>) -> ToolResult {
    let (marker, _) = load_marker(optm).map_err(io_err)?;
    let (target, ext) = match format {
        "pdf" => (PlotTarget::Pdf(PdfPage::A0), "pdf"),
        "hpgl" => (
            PlotTarget::Hpgl(PlotterSetup {
                paper_width: marker.material.width + 100.0,
                ..setup_from_json(None)
            }),
            "plt",
        ),
        other => return Err(format!("format must be \"pdf\" or \"hpgl\", got {other:?}")),
    };
    let path: PathBuf = out.map_or_else(|| optm.with_extension(ext), Path::to_path_buf);
    let mut i18n = I18n::new(Lang::En);
    i18n.add(rajutara_nest::i18n::catalog());
    let fmt = ReportFmt {
        unit: Unit::Cm,
        decimals: 2,
        decimal_comma: false,
    };
    let mut book = ShapeBook::default();
    let header = header_lines(&marker, &mut book, &utc_now_stamp(), &fmt, &|id| {
        i18n.get(id)
    });
    let job = marker_job(&marker, &mut book, &header).map_err(|e| e.to_string())?;
    let warnings = plot_marker(&job, &target, &path).map_err(|e| e.to_string())?;
    Ok(json!({ "written": path.display().to_string(), "writer_warnings": warnings.len() }))
}
