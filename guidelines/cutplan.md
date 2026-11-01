# Cut Plan (`optirust-cutplan`)

Turns an `Order` (garments per size and colour) into planned markers and Lays (stacks of plies), with consumption, cost and reports. Spec: `docs/spec/05-cutplan.md`. Data types are in `optirust-core` (`core-model.md`); behaviour here.

Flow: build a `CutPlan` (embed a Style with `optirust_io::embed_style`, fill `order`, `material`, `limits`, `costs`) -> `bundle_areas` -> `solve_plan` (estimated lengths) or `nest_plan` (real nested lengths) -> `apply_result` -> `results_view` / `cut_order` / `build_report` -> `optirust_io::save_cut_plan`.

## Order

`OrderLine.quantities` is in Size Table order. `deviation_pct` in `LayLimits` is the over-cut allowed per size and colour.

```rust sig
pub fn allowance(ordered: u32, deviation_pct: f64) -> u32
pub fn distribute_by_ratio(ratio: &[u32], total: u32) -> Result<Vec<u32>, CutPlanError>
pub fn line_total(line: &OrderLine) -> u64
pub fn size_totals(order: &Order, sizes: usize) -> Vec<u64>
pub fn grand_total(order: &Order) -> u64
pub fn import_order_csv(text: &str, sizes: &SizeTable) -> Result<OrderImport, CutPlanError>
```

## Solving lays

```rust sig
pub fn bundle_areas(style: &Style, material: &Material) -> BundleAreas
pub fn solve_plan(plan: &CutPlan, areas: &BundleAreas) -> Result<PlanResult, CutPlanError>
pub fn solve_with(plan: &CutPlan, areas: &BundleAreas, known: BTreeMap<Vec<u32>, NestedFact>) -> Result<PlanResult, CutPlanError>
pub fn apply_result(plan: &mut CutPlan, result: PlanResult)
pub fn solve(order: &Order, limits: &LayLimits, rules: PlyRules, lengths: &dyn LengthModel) -> Result<Solution, SolveError>
pub fn check_limits(limits: &LayLimits) -> Result<(), SolveError>
```

- `bundle_areas`: area of one Bundle per size (pieces of the Material, times quantity, times 2 for `pair`, with shrinkage). Marker length is estimated from it and `CostInputs.estimated_efficiency_pct`.
- `solve_plan`: markers and Lays using the nested lengths the plan already holds, estimates otherwise. Errors: `CutPlanError::RatioSize`, `NoPieces`, `BadMaterial`, or a `SolveError` (`check_limits` rejects `max_bundles` 0 or above 1000 and a non-positive `max_length`).
- `solve_with`: same with `known` nested facts keyed by ratio.
- `apply_result`: replaces `plan.markers` and `plan.lays`; ids come from `plan.ids`, never reused.
- `solve`: the pure planner behind it; `LengthModel` maps a ratio to a length (`FixedLength` for what-ifs and tests).
- `PlyRules::of(&material.ply_mode)`: Tubular, Faced and Folded need at least 2 plies; Faced plies come in pairs.

## Nesting the planned markers

```rust sig
pub fn nest_plan(plan: &CutPlan, areas: &BundleAreas, nester: &dyn RatioNester, cancelled: &(dyn Fn() -> bool + Sync), on_step: &dyn Fn(usize, usize)) -> Result<PlanResult, CutPlanError>
pub fn save_markers(plan: &CutPlan, dir: &Path, stem: &str) -> Result<Vec<PathBuf>, CutPlanError>
pub struct EngineNester {
pub enum NestStop {
Seconds(u32),
Iterations(u32),
pub const NEST_SEED_BASE: u64 = 1;
```

- `nest_plan`: solve, nest every distinct ratio with `nester`, re-solve once if a nested marker is longer than the table. `on_step(done, total)` reports progress; `cancelled` is polled before each marker. Runs on a worker thread.
- `EngineNester` is the real engine (`marker_for_ratio` then `optirust_nest::nest_marker`); marker `i` uses seed `NEST_SEED_BASE + i`. Use `NestStop::Iterations` in tests for determinism. A test double implements `RatioNester`.
- `save_markers` writes `<stem>-M<n>.optm` for each nested marker.

## Results and reports

```rust sig
pub fn results_view(plan: &CutPlan) -> ResultsView
pub fn cut_order(plan: &CutPlan) -> CutOrder
pub fn build_report(plan: &CutPlan, kind: ReportKind, ctx: &ReportContext<'_>) -> Report
pub fn import_rolls_csv(text: &str, local: &RollHeaders<'_>) -> Result<RollImport, CutPlanError>
```

- `results_view`: marker and Lay rows, per-cell coverage (`CoverageStatus`) and totals.
- `cut_order`: the Cut Order, each Ply drawn whole from one Fabric Roll (Roll Priority is the order of `plan.rolls`); reports `MissingFabric`.
- `build_report`: a `Report` of the chosen `ReportKind`; `Report::doc()` gives a `ReportDoc` for `optirust_io::write_report_pdf`; `write_csv_files` writes the CSV set.
- Errors are `CutPlanError` / `SolveError` enums; never unwrap on an Order or roll file.
