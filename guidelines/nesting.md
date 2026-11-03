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

`marker_for_ratio` builds a Marker from one planned ratio (used by Cut Plan). For hand-built markers use `MarkerSetup` and the commands, as `examples/nest_from_rjx.rs` of this repo does:

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

## Marker features the engine respects (1.4.0, 1.5.0)

`nest_marker` builds its items and fixed obstacles from the whole Marker, so these settings shape every run. All are undoable commands in `rajutara_nest::marker`.

```rust sig
pub fn set_pinned(ids: Vec<InstanceId>, pinned: bool) -> impl Command<Marker>
pub fn make_cluster(ids: Vec<InstanceId>, block: Option<FuseBlock>) -> impl Command<Marker>
pub fn delete_clusters(ids: Vec<InstanceId>) -> impl Command<Marker>
pub fn check_new(marker: &Marker, ids: &[InstanceId], block: bool) -> Result<(), ClusterError>
pub fn add_flaw(spec: FlawSpec) -> impl Command<Marker>
pub fn import_flaws(specs: Vec<FlawSpec>) -> impl Command<Marker>
pub fn remove_pieces_hitting_flaws(ids: Vec<InstanceId>) -> impl Command<Marker>
pub fn obstacles(marker: &Marker) -> Vec<(NestItem, Placed)>
pub fn insert_to_hole(marker: &Marker, book: &mut ShapeBook, id: InstanceId, delta: Vec2, pick: f64) -> Result<Pose, HoleError>
pub fn insert_command(id: InstanceId, pose: Pose) -> impl Command<Marker>
pub fn sequence(marker: &Marker, book: &mut ShapeBook) -> Vec<Cut>
pub fn optimize(marker: &Marker, book: &mut ShapeBook, opts: OptimizeOptions) -> Vec<CutStep>
pub fn set_cut_order(label: &'static str, steps: Vec<CutStep>) -> impl Command<Marker>
pub fn cut_job(marker: &Marker, book: &mut ShapeBook) -> Result<PlotJob, PlotError>
```

- `commands::set_pinned` (Fix on Marker): a pinned placed instance is a fixed obstacle even with `keep_placed: false`.
- `cluster::make_cluster` (Cluster Pieces; with `Some(FuseBlock)` a Block Fuse): check first with `cluster::check_new`. A cluster is nested as one item (`cluster::nesting_ring`, the border for a Fuse Block) and is never split; it only translates.
- `commands::add_flaw` / `import_flaws` (rows parsed by `flaws::parse_csv`): `flaws::obstacles` makes every flaw a fixed obstacle, so neither Auto Nesting nor Place All puts a piece on one.
- `hole::insert_to_hole(&marker, &mut book, id, delta, pick) -> Result<Pose, HoleError>` finds the nearest free spot that fits a dragged placed piece (any allowed orientation, then a Shoot along `delta`); apply it with `hole::insert_command`. `pick` is the pointer pick radius in mm: a hole with no room to spare counts within `pick` or `HOLE_SNAP_MM` of the drop (1.5.1). Refused for a cluster member or when nothing fits within reach.
- `cutting::optimize` orders the cuts nearest-neighbour from the table corner (`OptimizeOptions { from_top, choose_starts }`); `set_cut_order` stores the steps. `output::cutfile::cut_job` follows `Marker.cut_order`, each start vertex and knife direction (clockwise by default), and cuts a Fuse Block as its border alone; write it with `cutfile::write_cutfile`.
- Flaws and bump lines block placement but never add to the used length or the Efficiency.

### Bump/ Cut Lines, Marker texts, Substitute Size (1.5.0)

```rust sig
pub fn set_bump_lines(rows: Vec<LineRow>, settings: CutLineSettings) -> impl Command<Marker>
pub fn add_bump_line(axis: LineAxis, at: f64) -> impl Command<Marker>
pub fn move_bump_line(id: BumpLineId, at: f64) -> impl Command<Marker>
pub fn rows(marker: &Marker) -> Vec<LineRow>
pub fn walls(marker: &Marker) -> impl Iterator<Item = &BumpLine>
pub fn end_nesting(m: &mut Marker, book: &mut ShapeBook)
pub fn cut_segments(marker: &Marker, cuts: &[BezPath], used_length: f64) -> Vec<Line>
pub struct LineRow {
pub id: Option<BumpLineId>,
pub fn add_text(spec: TextSpec, all_sizes: bool) -> impl Command<Marker>
pub fn edit_text(id: MarkerTextId, spec: TextSpec) -> impl Command<Marker>
pub fn move_text(id: MarkerTextId, delta: Vec2) -> impl Command<Marker>
pub fn remove_text(id: MarkerTextId) -> impl Command<Marker>
pub fn remove_all_texts() -> impl Command<Marker>
pub fn views(marker: &Marker) -> Vec<TextView>
pub struct TextSpec {
pub on: Option<InstanceId>,
pub struct FlawSpec {
pub x: Option<(f64, f64)>,
pub y: Option<(f64, f64)>,
pub fn parse_csv(text: &str) -> Result<Vec<FlawSpec>, CsvError>
pub fn substitutions(marker: &Marker, ids: &[InstanceId]) -> Vec<Substitution>
pub fn substitute_size(swaps: Vec<(InstanceId, InstanceId)>) -> impl Command<Marker>
```

- Bump lines (`commands::set_bump_lines` from the Bump/ Cut Lines dialog rows, `add_bump_line`, `move_bump_line`; helpers in `marker::bump`): every line not marked `no_bump` is a wall (`bump::walls`, `bump::obstacles`) that Auto Nesting, the Place commands and Shoot stop at. With `CutLineSettings.auto_vertical`, `bump::end_nesting` adds a vertical line at the used length after Auto Nesting. The cut file cuts the `cut` lines, minus what lies inside pieces (`bump::cut_segments`), with the top and bottom overcut; the plot draws cut lines solid and bump lines dashed.
- Marker texts (`commands::add_text` with `TextSpec`; `all_sizes` puts it on every placed copy of the piece): `TextSpec.at` and `angle` are on the table; a text with `on` is stored in that instance's frame and moves and turns with it (`texts::views` gives where each one shows). Texts are plotted, never cut.
- Flaws come from the dialog or from `flaws::parse_csv` (`x_from,y_from,x_to,y_to` in mm; a bad line refuses the whole list with `CsvError`); `FlawSpec.x` / `y` = `None` is a Whole Length / Whole Width Flaw.
- Substitute Size For Selected Pieces: `commands::substitutions` pairs each selected placed instance with a waiting copy of the same piece in the next size (or a reason it cannot); `substitute_size` swaps the pairs in one undo step.

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
- Reports: `report_data`, then `write_xlsx_report` (File › Report to Excel: one worksheet per section, General, Order, Pieces and Notches, built by `sheets`), or `report_doc` then `rajutara_io::write_report_pdf` (Print). The CSV report writer of 1.4.0 is gone.

```rust sig
pub fn sheets(data: &ReportData) -> [(String, Vec<Vec<String>>); 4]
pub fn write_xlsx_report(data: &ReportData, path: &Path) -> Result<(), IoError>
pub fn report_doc(data: ReportData, drawing: Option<PlotJob>) -> ReportDoc
```
