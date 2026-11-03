# File I/O (`rajutara-io`)

Every public loader and writer, re-exported at the crate root: `use rajutara_io::{load_style, ...}`. Failures are `IoError` (`Io`, `Format`, `NewerVersion`, `Invalid(Vec<Issue>)`, `Dxf`, `Refused`, `Cancelled`); anything an importer or exporter skipped or approximated comes back as `Vec<ImportIssue>` (`severity`, `location`, `message`). Show the issues; never discard them.

## Native files

`.rjx` Style, `.rjm` Marker, `.rjc` Cut Plan: a zip container with `manifest.json` (`format`, `version`, `generator`) and `document.json`, format version 2. A file saved by a newer format is refused (`NewerVersion`). Loaders validate: errors refuse the file (`IoError::Invalid`), warnings are returned. The loaders read the container whatever the extension, so the pre-rename optirust `.optx`, `.optm`, `.optc` files still load; the app saves them under the new extension. Fields added in 1.5.0 (Marker clusters, cut order, bump lines, flaws, texts; vertex names; Equal Segments Groups, Fabric Pattern, zones, Spread, Fixed Placement, Stripe Groups, piece colour, one-line label) are optional and did not bump the format: a 1.4.0 build opens such a file and drops them on re-save, and refuses a Style that uses Stack Along Line.

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
- `embed_style` loads a `.rjx` as an `EmbeddedStyle` for a Marker or Cut Plan, remembering its path and CRC-32 (`source_crc32`) so a changed source is detectable; `style_crc32` recomputes it.

## DXF and RUL

ASTM D6673 and AAMA-292 DXF with ASTM RUL grade rules; plain DXF import. The importer reads the `.rul` file named in the DXF's `Grade Rule Table:` next to the DXF. Import units come from the file (`Units:` text, then `$INSUNITS`, else cm) unless `DxfImportOptions.units` forces one. Files over 64 MiB are refused.

```rust sig
pub fn import_dxf(path: &Path, opts: &DxfImportOptions) -> Result<(Style, Vec<ImportIssue>), IoError>
pub fn import_dxf_cancellable(path: &Path, opts: &DxfImportOptions, cancelled: &dyn Fn() -> bool) -> Result<(Style, Vec<ImportIssue>), IoError>
pub fn import_dxf_with_rul_cancellable(path: &Path, rul: Option<&Path>, opts: &DxfImportOptions, cancelled: &dyn Fn() -> bool) -> Result<(Style, Vec<ImportIssue>, DxfFileInfo), IoError>
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
pub line_mode: LineKind,
pub struct DxfFileInfo {
pub unit_mm: f64,
pub flavor: DxfFlavor,
pub layers: BTreeMap<u16, usize>,
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
- `import_dxf_with_rul_cancellable`: the import wizard's call. `rul: Some(path)` replaces the lookup of the file named by `Grade Rule Table:`; the extra `DxfFileInfo` says what the file declared (unit, dialect, entities per layer). `line_mode: LineKind::Sew` (default) keeps the boundary as the sewing line; `Cut` keeps it as the cutting line and turns layer 14 into an internal Sew line.
- `parse_dxf`: in-memory import; pass the `.rul` text yourself. The Style is named `Imported` when the file has no `Style Name:`.
- `export_dxf` / `render_dxf`: write or render ASTM (default) or AAMA. `DxfFlavor::Auto` and `Plain` are refused on export. `GradingExport::RuleFile` writes base-size blocks plus a `.rul` next to the DXF, `GradedNest` one block per size, `BaseSizeOnly` no grading. Set `created` from `utc_now_stamp()`; the default is the epoch.
- `read_rul` returns `(rule table, SIZE LIST, SAMPLE SIZE, issues)`; deltas are mm, cumulative from the sample size. `write_rul` renders the file text; `RulOptions { metric, positive_rule_numbers, created }`.

## Plots: HPGL/2 and PDF

A `PlotJob` is a `sheet: Rect` plus `PlotItem`s (`kind: PlotKind`, a `BezPath`, optional text). Build it with `rajutara_pds::plot::job::plot_job` (pieces) or `rajutara_nest::output::plot::marker_job` (marker).

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

## Excel workbooks (`rajutara_io::xlsx`, 1.5.0)

```rust sig
pub fn write_xlsx(rows: &[Vec<String>], sheet: &str, path: &Path) -> Result<(), IoError>
pub fn write_workbook(sheets: &[Sheet<'_>], path: &Path) -> Result<(), IoError>
pub fn render_workbook(sheets: &[Sheet<'_>]) -> Result<Vec<u8>, IoError>
pub struct Sheet<'a> {
pub name: &'a str,
pub rows: &'a [Vec<String>],
```

- `write_xlsx` (re-exported at the root) writes one worksheet; `xlsx::write_workbook` several, in order; `render_workbook` returns the bytes. ECMA-376 SpreadsheetML, no styles, written through `write_atomic`. Used by File › Report to Excel.
- Cells: a finite number with `.` as decimal mark becomes a number cell, except with a leading zero or more than 15 digits (codes and ids stay text); everything else is an inline string.
- Sheet names: `[ ] : * ? / \` become `_`, cut to 31 characters, empty becomes `Sheet<n>`. `IoError::Refused` for no sheet, two names equal without case, or more than 1 048 576 rows or 16 384 columns.
- Deterministic: fixed zip timestamps and part order, so the same rows give the same bytes (snapshot-testable).
- The Marker's Report to Excel is `rajutara_nest::report::write_xlsx_report` (one sheet per section, `report::sheets`; see `nesting.md`); PDS's is in `rajutara_pds::tools::reports`. Cut Plan reports stay CSV and PDF.

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
