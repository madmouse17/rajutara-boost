# Grading

Rule-based grading (ADR-0004): geometry for every size is **derived** from the Base Size and Grade Rules; sizes are never stored as separate pieces. Spec: `docs/spec/03-grading.md`. Rule math lives in `rajutara-core::grading`; `rajutara-grading` decides which rules to write and always asks core for positions.

## Data

- `SizeTable { sizes, base }`: smallest first, one Base Size.
- `GradeRule { deltas }`: one `Vec2` per size, cumulative from the Base Size, `deltas[base] == Vec2::ZERO`.
- `Piece.grading: BTreeMap<PointId, GradeRule>`: the Grade Points. Applies to contour vertices, internal-line vertices, grainline ends, drill holes, annotations.
- `Style.rule_table`: numbered rules shared by points (ASTM grade rule numbers, positive or negative, unique).
- A contour vertex with no entry is graded in proportion to its neighbouring Grade Points; other points do not move.
- Notches placed `Between` two vertices (`NotchRelation`) follow the vertices; per-size notch values live in `NotchPlace.per_size`.

## Deriving sizes (`rajutara-core`)

```rust sig
pub fn piece_for_size(piece: &Piece, sizes: &SizeTable, size: usize) -> Piece
pub fn graded_nest(piece: &Piece, sizes: &SizeTable) -> Vec<Piece>
pub fn incremental(rule: &GradeRule, sizes: &SizeTable) -> Vec<Vec2>
```

- `piece_for_size`: the piece in size index `size`. The Base Size (or an index outside the table) returns a clone. Any other size returns a derived piece with empty `grading`, so grading it again changes nothing.
- `graded_nest`: all sizes in Size Table order.
- `incremental`: the size-to-size step view of a rule, zero at the Base Size; for display, not storage.

```rust
let style = rajutara_io::load_style(path)?.0;
let m = style.size_table.index_of("M").ok_or("no size M")?;
let l = style.size_table.index_of("L").ok_or("no size L")?;
let front = &style.pieces[0];
let width = |size| piece_for_size(front, &style.size_table, size).contour.bounding_box().width();
println!("L is {:.1} mm wider than M", width(l) - width(m));
```

## Editing grading (`rajutara-grading`)

Each operation takes `&Style` and returns a `GradeCommand` (`Edit<Box<dyn FnOnce(&mut Style) -> Result<(), CommandError>>>`) for `Document<Style>::execute`, so it is undoable and validated. `PointRef` is `(PieceId, PointId)`.

```rust sig
pub fn insert_sizes(style: &Style, before: usize, new: Vec<Size>) -> Result<GradeCommand, GradingError>
pub fn delete_size(style: &Style, index: usize) -> Result<GradeCommand, GradingError>
pub fn rename_sizes(style: &Style, names: Vec<String>) -> Result<GradeCommand, GradingError>
pub fn set_base_size(style: &Style, new_base: usize) -> Result<GradeCommand, GradingError>
pub fn regrade_by_break(style: &Style, pieces: &[PieceId]) -> Result<GradeCommand, GradingError>
pub fn add_rule(style: &Style, name: &str, rule: GradeRule) -> Result<GradeCommand, GradingError>
pub fn delete_rule(style: &Style, number: i32) -> Result<GradeCommand, GradingError>
pub fn apply_rule(style: &Style, number: i32, targets: &[PointRef]) -> Result<GradeCommand, GradingError>
pub fn matching_rule_number(table: &RuleTable, rule: &GradeRule) -> Option<i32>
pub fn grading_issues(style: &Style) -> Vec<GradingIssue>
```

- `insert_sizes` / `append_sizes`: add sizes; every existing rule gets deltas for them.
- `set_base_size`: change the Base Size; every rule is re-expressed from it (new delta = old delta minus old delta of the new base), so every size keeps its geometry. `delete_size` refuses the Base Size.
- `regrade_by_break`: sizes between two consecutive breaks (`Size.is_break`; smallest, largest and Base Size always count) get the linear interpolation of the breaks' deltas, for every Grade Point of the given pieces.
- `apply_rule`: set a numbered rule on the target points.
- `grading_issues`: core validation plus rule-length and name checks. Export refuses while it is non-empty (except Base Size only).
- Always check the rule length: `deltas.len()` must equal the number of sizes (`GradingError` otherwise).

## Export

```rust sig
pub fn run_export(style: &Style, req: &ExportRequest, created: (String, String)) -> Result<ExportDone, ExportError>
pub fn run_rul_export(style: &Style, path: &Path, positive_rule_numbers: bool, metric: bool, created: (String, String)) -> Result<ExportDone, ExportError>
pub struct ExportRequest {
pub path: PathBuf,
pub grading: GradingExport,
pub visible_sizes_only: bool,
pub positive_rule_numbers: bool,
pub metric: bool,
pub pieces: Option<Vec<PieceId>>,
```

- `run_export` writes an ASTM DXF (plus `.rul` next to it for `GradingExport::RuleFile`) after the grading gate.
- `run_rul_export` writes the Rule Table only. `created` is `rajutara_io::utc_now_stamp()`.

Verify grading with a property test (core `grading_props.rs`): the Base Size is unchanged, a zero rule moves nothing, `piece_for_size` of a derived piece is idempotent.
