# Nesting (`rajutara-nest`)

Headless irregular strip packing: no-fit polygons, bottom-left fill, simulated annealing over the piece order, compaction. No UI types in the engine API. Spec: `docs/spec/04-marker.md`.

## One call: `nest_marker`

Auto-nests every instance of a `Marker` that has no pose and returns a new `Marker` with poses applied plus its metrics. The input is untouched.

```rust sig
pub fn nest_marker(marker: &Marker, opts: &NestMarkerOptions, progress: &(dyn Fn(Progress) -> Control + Sync)) -> Result<NestedMarker, NestError>
pub struct NestMarkerOptions {
pub stop: Stop,
pub target_efficiency: Option<f64>,
pub seed: u64,
pub threads: usize,
pub unlimited_length: bool,
pub keep_placed: bool,
pub struct NestedMarker {
pub marker: Marker,
pub length: f64,
pub efficiency_pct: f64,
pub unplaced: usize,
pub enum NestError {
NothingToNest,
Panicked(String),
pub enum Stop {
Time(Duration),
Iterations(u32),
pub enum Control {
Continue,
Cancel,
pub struct Progress {
pub elapsed: Duration,
pub best_length: f64,
pub best_efficiency: f64,
```

- `NestMarkerOptions::default()`: `Stop::Time(20 s)`, no `target_efficiency`, `seed: 1`, `threads: 0` (available parallelism, capped at 4), `unlimited_length: true`, `keep_placed: true`.
- `keep_placed: true` nests only waiting instances around the already placed ones; `false` renests everything. All instances placed and `keep_placed` on gives `NestError::NothingToNest`; clear poses (or `keep_placed: false`) to renest.
- `unplaced` counts instances that did not fit the Material width or have no usable shape. Check it is 0 before you plot.
- `length` and `efficiency_pct` are `04-marker.md` §8.1 figures: used length in mm, placed area over width times used length.
- `progress` is called about every 50 ms; return `Control::Cancel` to stop and keep the best packing so far.

## Determinism and time limits

- `Stop::Iterations(n)` is fully deterministic for one seed: use it in tests, benchmarks and anything that must repeat byte for byte.
- `Stop::Time(d)` counts wall-clock from the start, shape preparation included. The search stops at 90 % of `d`, compaction at 97 %. The result depends on machine speed, so the same seed can give different layouts on different machines.
- Cut Plan markers use seed `NEST_SEED_BASE + i` for marker `i`, so re-running a plan repeats under `Stop::Iterations`.
- A panic inside the engine is caught and returned as `NestError::Panicked`; the Marker is untouched (ADR-0009).
- Run nesting on a worker thread (`std::thread`), never on the UI thread.

## Building a Marker

`marker_for_ratio` builds a Marker from one planned ratio (used by Cut Plan). For hand-built markers use `MarkerSetup` and the commands, as `crates/rajutara/examples/nest_from_optx.rs` does:

```rust sig
pub fn marker_for_ratio(style: &EmbeddedStyle, material: &Material, max_length: f64, ratio: &[u32], name: &str) -> Marker
pub fn new(name: impl Into<String>, buffer_mm: f64) -> Self
pub fn into_marker(self) -> Result<Marker, Vec<MarkerSetupError>>
pub fn from_marker(marker: &Marker) -> Self
pub fn add_style(embedded: EmbeddedStyle) -> impl Command<Marker>
pub fn set_setup(setup: MarkerSetup) -> impl Command<Marker>
pub fn set_request(style: StyleRefId, size: String, bundles: u32, direction: BundleDirection) -> impl Command<Marker>
pub fn apply_nest(poses: Vec<(InstanceId, Option<Pose>)>) -> impl Command<Marker>
```

Order matters: `MarkerSetup::new(name, buffer)` then set `width`; `into_marker`; `Document::new`; `add_style`; `set_setup` with `material` set to the material of the pieces (only pieces of that material are instanced); one `set_request` per size. Each command syncs the instances.

## Measuring and plotting

```rust sig
pub fn metrics(marker: &Marker, book: &mut ShapeBook) -> Metrics
pub fn conflicts(marker: &Marker, book: &mut ShapeBook) -> Conflicts
pub fn overlap_count(&self) -> usize
pub fn header_lines(marker: &Marker, book: &mut ShapeBook, date: &(String, String), fmt: &ReportFmt, tr: Tr<'_>) -> Vec<String>
pub fn marker_job(marker: &Marker, book: &mut ShapeBook, header: &[String]) -> Result<PlotJob, PlotError>
pub fn plot_marker(job: &PlotJob, target: &PlotTarget, path: &Path) -> Result<Vec<ImportIssue>, PlotError>
pub fn setup_from_json(stored: Option<&serde_json::Value>) -> PlotterSetup
pub enum PlotTarget {
Pdf(PdfPage),
Hpgl(PlotterSetup),
```

- `ShapeBook::default()` caches piece shapes; reuse one across `metrics`, `conflicts`, `header_lines`, `marker_job`.
- `Metrics` has `requested`, `placed`, `used_length`, `efficiency_pct`, `bundles_placed`, `fabric_cost` and more. `Conflicts` lists `overlapping`, `outside`, `in_margin` instance sets; a good marker has `overlap_count() == 0` and empty `in_margin`.
- `marker_job` returns `PlotError::NothingPlaced` when no instance has a pose. `plot_marker` returns `PlotError::TooWide` for HPGL when the marker is wider than the roll: set `PlotterSetup.paper_width` to fabric width plus margin.
- `header_lines` needs a translator: `let mut i18n = I18n::new(Lang::En); i18n.add(rajutara_nest::i18n::catalog());` then `&|id| i18n.get(id)`. The i18n loaders are process-global; do not switch language in parallel tests.
- Reports: `report_data` then `write_csv_report`, or `report_doc` then `rajutara_io::write_report_pdf`.
