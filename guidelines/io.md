# File I/O (`optirust-io`)

Every public loader and writer, re-exported at the crate root: `use optirust_io::{load_style, ...}`. Failures are `IoError` (`Io`, `Format`, `NewerVersion`, `Invalid(Vec<Issue>)`, `Dxf`, `Refused`, `Cancelled`); anything an importer or exporter skipped or approximated comes back as `Vec<ImportIssue>` (`severity`, `location`, `message`). Show the issues; never discard them.

## Native files

`.optx` Style, `.optm` Marker, `.optc` Cut Plan: a zip container with `manifest.json` (`format`, `version`, `generator`) and `document.json`. A file saved by a newer optirust is refused (`NewerVersion`). Loaders validate: errors refuse the file (`IoError::Invalid`), warnings are returned.

```rust sig
pub fn load_style(path: &Path) -> Result<(Style, Vec<Issue>), IoError>
pub fn save_style(style: &Style, path: &Path) -> Result<(), IoError>
pub fn load_marker(path: &Path) -> Result<(Marker, Vec<Issue>), IoError>
pub fn save_marker(marker: &Marker, path: &Path) -> Result<(), IoError>
pub fn load_cut_plan(path: &Path) -> Result<(CutPlan, Vec<Issue>), IoError>
pub fn save_cut_plan(plan: &CutPlan, path: &Path) -> Result<(), IoError>
pub fn embed_style(path: &Path, id: StyleRefId) -> Result<(EmbeddedStyle, Vec<Issue>), IoError>
pub fn style_crc32(path: &Path) -> Result<u32, IoError>
```

- `load_*` read and validate; `save_*` write atomically (see `write_atomic`).
- `embed_style` loads a `.optx` as an `EmbeddedStyle` for a Marker or Cut Plan, remembering its path and CRC-32 (`source_crc32`) so a changed source is detectable; `style_crc32` recomputes it.

## DXF and RUL

ASTM D6673 and AAMA-292 DXF with ASTM RUL grade rules; plain DXF import. The importer reads the `.rul` file named in the DXF's `Grade Rule Table:` next to the DXF. Import units come from the file (`Units:` text, then `$INSUNITS`, else cm) unless `DxfImportOptions.units` forces one. Files over 64 MiB are refused.

```rust sig
pub fn import_dxf(path: &Path, opts: &DxfImportOptions) -> Result<(Style, Vec<ImportIssue>), IoError>
pub fn import_dxf_cancellable(path: &Path, opts: &DxfImportOptions, cancelled: &dyn Fn() -> bool) -> Result<(Style, Vec<ImportIssue>), IoError>
pub fn parse_dxf(bytes: &[u8], rul_text: Option<&str>, opts: &DxfImportOptions) -> Result<(Style, Vec<ImportIssue>), IoError>
pub fn export_dxf(style: &Style, path: &Path, opts: &DxfExportOptions) -> Result<Vec<ImportIssue>, IoError>
pub fn render_dxf(style: &Style, opts: &DxfExportOptions) -> Result<DxfExport, IoError>
pub struct DxfExport {
pub dxf: Vec<u8>,
pub rul: Option<String>,
pub issues: Vec<ImportIssue>,
pub struct DxfImportOptions {
pub flavor: DxfFlavor,
pub units: Option<Unit>,
pub tolerance: f64,
pub build_sizes: bool,
pub struct DxfExportOptions {
pub flavor: DxfFlavor,
pub metric: bool,
pub grading: GradingExport,
pub created: (String, String),
pub enum GradingExport {
pub fn read_rul(text: &str) -> Result<(RuleTable, Vec<String>, String, Vec<ImportIssue>), IoError>
pub fn write_rul(table: &RuleTable, sizes: &SizeTable, opts: &RulOptions) -> String
pub fn write_rul_atomic(path: &Path, table: &RuleTable, sizes: &SizeTable, opts: &RulOptions) -> Result<(), IoError>
```

- `import_dxf`: file in, `Style` out. `DxfImportOptions::default()` is the dialog default (`flavor: Auto`, tolerance 0.5 mm, `build_sizes: true`).
- `import_dxf_cancellable`: same, for a worker thread; `cancelled` is polled between pieces; returns `IoError::Cancelled`.
- `parse_dxf`: in-memory import; pass the `.rul` text yourself. The Style is named `Imported` when the file has no `Style Name:`.
- `export_dxf` / `render_dxf`: write or render ASTM (default) or AAMA. `DxfFlavor::Auto` and `Plain` are refused on export. `GradingExport::RuleFile` writes base-size blocks plus a `.rul` next to the DXF, `GradedNest` one block per size, `BaseSizeOnly` no grading. Set `created` from `utc_now_stamp()`; the default is the epoch.
- `read_rul` returns `(rule table, SIZE LIST, SAMPLE SIZE, issues)`; deltas are mm, cumulative from the sample size. `write_rul` renders the file text; `RulOptions { metric, positive_rule_numbers, created }`.

## Plots: HPGL/2 and PDF

A `PlotJob` is a `sheet: Rect` plus `PlotItem`s (`kind: PlotKind`, a `BezPath`, optional text). Build it with `optirust_pds::plot::job::plot_job` (pieces) or `optirust_nest::output::plot::marker_job` (marker).

```rust sig
pub fn render_hpgl(job: &PlotJob, setup: &PlotterSetup) -> Result<(Vec<u8>, Vec<ImportIssue>), IoError>
pub fn plot_hpgl(job: &PlotJob, setup: &PlotterSetup, path: &Path) -> Result<Vec<ImportIssue>, IoError>
pub fn render_pdf(job: &PlotJob, page: PdfPage) -> Result<(Vec<u8>, Vec<ImportIssue>), IoError>
pub fn plot_pdf(job: &PlotJob, page: PdfPage, path: &Path) -> Result<Vec<ImportIssue>, IoError>
pub enum PdfPage {
pub struct PlotterSetup {
pub paper_width: f64,
pub rotate_90: bool,
pub calibration: kurbo::Vec2,
pub pens: Vec<PenAssignment>,
```

- `plot_hpgl` / `render_hpgl`: HPGL/2 for a roll plotter. `IoError::Refused` when the sheet is wider than `paper_width`.
- `plot_pdf` / `render_pdf`: `PdfPage::A4Tiled` (1:1 tiles plus an assembly map), `A0` (one page if it fits), `A4Fit` (whole sheet scaled onto one A4 page). Refused above 10 000 tiles.
- `PlotterSetup::default()` puts every element on pen 1; for markers use `setup_from_json(None)` (`rotate_90` off) and set `paper_width` to the fabric width plus margin.

## Reports and CSV

```rust sig
pub fn render_report_pdf(doc: &ReportDoc) -> Result<(Vec<u8>, Vec<ImportIssue>), IoError>
pub fn write_report_pdf(doc: &ReportDoc, path: &Path) -> Result<Vec<ImportIssue>, IoError>
pub fn write_csv<R: Serialize>(rows: &[R], path: &Path) -> Result<(), IoError>
pub fn read_csv_rows(text: &str) -> Result<Vec<Vec<String>>, IoError>
```

- `ReportDoc { title, sections }`; a `ReportSection` has a heading, facts and an optional `ReportTable`.
- `write_csv`: struct rows get a header; `Vec<String>` rows are written as they are. Cells that would be read as formulas are quoted with a leading `'`.
- `read_csv_rows` strips a UTF-8 BOM.

## File-system helpers

```rust sig
pub fn write_atomic(path: &Path, bytes: &[u8], keep_backup: bool) -> Result<(), IoError>
pub fn autosave_path(doc: &Path) -> PathBuf
pub fn newer_autosave(doc: &Path) -> Option<PathBuf>
pub fn utc_stamp(unix_secs: i64) -> (String, String)
pub fn utc_now_stamp() -> (String, String)
```

- `write_atomic`: temp file beside `path`, then rename. A reader or a crash sees the old file or the new one. Creates no directories; `keep_backup` copies the old file to `<path>.bak` first.
- `autosave_path`: `<stem>.autosave.<ext>` beside the document; `newer_autosave` returns it when it is newer than the document.
- `utc_stamp` / `utc_now_stamp`: `("dd-mm-yyyy", "hh:mm")` in UTC, the header stamp of DXF and reports.

Rules when you add a format or writer: return `Result<_, IoError>`, write through `write_atomic`, record every approximation as an `ImportIssue`, and cover DXF/HPGL output with `insta` snapshots.
