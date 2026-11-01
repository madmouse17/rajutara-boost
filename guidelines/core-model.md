# Core model (`optirust-core`)

Condensed from `docs/spec/01-core-model.md`. Everything is re-exported at the crate root: `use optirust_core::{Style, Piece, ...}`. Geometry types are `kurbo` (`Point`, `Vec2`, `Rect`, `BezPath`), re-exported.

Unit: mm, `f64`. Ids are typed newtypes (`PieceId`, `PointId`, `NotchId`, ...) handed out by an `IdAlloc` that lives in the document (`style.ids`, `marker.ids`, `plan.ids`): `ids.next()` infers the id type from where it is used. Never invent or reuse an id.

## Style

A `Style` is the unit a `.optx` file saves. `pieces` order is the Pieces Window, Plot and export order. Pieces are `Arc<Piece>`; mutate with `Arc::make_mut` or through a `Command`.

```rust sig
pub struct Style {
pub name: String,
pub author: String,
pub notes: String,
pub size_table: Arc<SizeTable>,
pub rule_table: Arc<RuleTable>,
pub pieces: Vec<Arc<Piece>>,
pub default_material: String,
pub ids: IdAlloc,
pub fn new(name: impl Into<String>) -> Self
pub fn piece(&self, id: PieceId) -> Option<&Arc<Piece>>
pub fn piece_mut(&mut self, id: PieceId) -> Result<&mut Piece, CommandError>
```

`Style::new` gives one size "M" as Base Size, no pieces, default material "Main".

## Piece and Contour

`Piece.contour` is the stored outline; `Piece.line` says whether it is the sewing or the cutting line, the other is derived from the per-vertex `SeamAllowance`.

```rust sig
pub struct Piece {
pub id: PieceId,
pub name: String,
pub quantity: u16,
pub pair: bool,
pub material: String,
pub line: LineKind,
pub contour: Contour,
pub mirror: Option<MirrorLine>,
pub grainline: Grainline,
pub internals: Vec<InternalLine>,
pub notches: Vec<Notch>,
pub drill_holes: Vec<DrillHole>,
pub annotations: Vec<Annotation>,
pub grading: BTreeMap<PointId, GradeRule>,
pub cut: CutRules,
pub fn new(ids: &mut IdAlloc, name: impl Into<String>, contour: Contour) -> Self
pub fn point(&self, id: PointId) -> Option<Point>
pub fn sewing_line(&self) -> DerivedLine
pub fn cutting_line(&self) -> DerivedLine
pub struct Contour {
pub points: Vec<Vertex>,
pub shapes: Vec<SegmentShape>,
pub struct Vertex {
pub id: PointId,
pub at: Point,
pub kind: PointKind,
pub seam: Option<SeamAllowance>,
pub fn new(id: PointId, at: Point, kind: PointKind) -> Self
pub enum PointKind {
pub struct SeamAllowance {
pub width: f64,
pub end_width: Option<f64>,
pub fn bounding_box(&self) -> Rect
pub fn index_of(&self, id: PointId) -> Option<usize>
pub fn length_between(&self, from: usize, to: usize) -> f64
```

Rules: segment `i` runs from `points[i]` to `points[(i + 1) % n]`; `shapes.len() == points.len()`; `PointKind` is `Turn` or `Curve`; `SegmentShape::FromPoints` is the plain segment. A `Vertex.seam` starts a Seam Allowance that runs forward to the next vertex that has its own. `Piece::new` allocates the piece id plus the two grainline point ids and adds a default vertical grainline.

## Notch, Grainline

```rust sig
pub struct Notch {
pub shape: NotchShape,
pub depth: f64,
pub width: f64,
pub at: NotchAt,
pub fn new(id: NotchId, at: NotchAt) -> Self
pub enum NotchAt {
Point(PointId),
Between {
pub struct NotchPlace {
pub relation: NotchRelation,
pub value: f64,
pub per_size: Vec<f64>,
pub struct Grainline {
pub start: PiecePoint,
pub end: PiecePoint,
pub initial_angle: f64,
```

`NotchAt::Point` sits on a vertex; `NotchAt::Between { prev, next, place }` sits along the contour, `NotchRelation::Proportional | FromPrevious | FromNext`.

## SizeTable, GradeRule, RuleTable

Sizes are positional: `GradeRule.deltas[i]` is the displacement of a point from the Base Size to size `i`, **cumulative from the Base Size**, and `deltas[base]` is always `Vec2::ZERO`. Markers and Orders refer to sizes by name. Size names are unique, non-empty, no whitespace.

```rust sig
pub struct SizeTable {
pub sizes: Vec<Size>,
pub base: usize,
pub fn single(name: impl Into<String>) -> Self
pub fn index_of(&self, name: &str) -> Option<usize>
pub struct Size {
pub name: String,
pub visible: bool,
pub fn new(name: impl Into<String>) -> Self
pub struct GradeRule {
pub deltas: Vec<Vec2>,
pub fn zero(sizes: usize) -> Self
pub struct RuleTable {
pub rules: Vec<NumberedRule>,
pub struct NumberedRule {
pub number: i32,
pub rule: GradeRule,
```

A point in `Piece.grading` (keyed by `PointId`) is a Grade Point. A contour vertex without an entry is graded in proportion to its neighbours; other points are not graded.

## Marker

An arrangement of piece instances on a fabric of given width. `Marker.styles` holds `EmbeddedStyle` copies; `requests` say how many Bundles of which size; `pieces` has one `MarkerPiece` per instance, `pose: None` while waiting in the Piece Bar.

```rust sig
pub struct Marker {
pub name: String,
pub material: Material,
pub max_length: f64,
pub plies: u32,
pub margins: Margins,
pub buffer: f64,
pub styles: Vec<EmbeddedStyle>,
pub requests: Vec<SizeRequest>,
pub pieces: Vec<MarkerPiece>,
pub struct Material {
pub name: String,
pub width: f64,
pub ply_mode: PlyMode,
pub struct EmbeddedStyle {
pub source_crc32: u32,
pub style: Arc<Style>,
pub struct SizeRequest {
pub size: String,
pub bundles: u32,
pub direction: BundleDirection,
pub struct MarkerPiece {
pub bundle: u32,
pub mirrored: bool,
pub pose: Option<Pose>,
pub struct Pose {
pub at: Point,
pub rotation: f64,
pub flip: Flip,
```

`Pose`: the piece is turned so its grainline points along +X, mirrored if `mirrored`, then `rotation` and `flip` about the grainline start, which lands on `at`. `BundleDirection` is `OneWay | Opposite | Any`; `Flip` is `None | X | Y`.

## CutPlan

The Lays that cut one `Order` from one `Material`. Behaviour is in `optirust-cutplan` (`cutplan.md`).

```rust sig
pub struct Order {
pub lines: Vec<OrderLine>,
pub struct OrderLine {
pub colour: String,
pub quantities: Vec<u32>,
pub struct CutPlan {
pub style: EmbeddedStyle,
pub order: Order,
pub material: Material,
pub limits: LayLimits,
pub costs: CostInputs,
pub markers: Vec<PlannedMarker>,
pub lays: Vec<Lay>,
pub rolls: Vec<FabricRoll>,
pub struct LayLimits {
pub min_plies: u32,
pub max_plies: u32,
pub max_length: f64,
pub max_bundles: u32,
pub deviation_pct: f64,
pub struct PlannedMarker {
pub ratio: Vec<u32>,
pub efficiency_pct: f64,
pub marker: Option<Arc<Marker>>,
pub struct Lay {
pub marker: PlannedMarkerId,
pub plies: u32,
```

`OrderLine.quantities` and `PlannedMarker.ratio` are in the Style's Size Table order.

## Validation

Validate before you trust a document and refuse on `Severity::Error`; warnings ride along. Loaders in `optirust-io` already do this.

```rust sig
pub enum Severity {
pub struct Issue {
pub severity: Severity,
pub piece: Option<PieceId>,
pub code: IssueCode,
pub fn validate_style(style: &Style) -> Vec<Issue>
pub fn validate_piece(piece: &Piece, sizes: &SizeTable) -> Vec<Issue>
pub fn validate_marker(marker: &Marker) -> Vec<Issue>
pub fn validate_cut_plan(plan: &CutPlan) -> Vec<Issue>
```

## Undo: Command, Document

Every edit of a UI document is a `Command<D>` run through `Document<D>::execute`; it is undoable and rejects results that add validation errors. `Edit { label, f }` is the closure-backed command; `label` is a Fluent id such as `"cmd-add-notch"`.

```rust sig
pub trait Command<D> {
pub struct Edit<F> {
pub struct Document<D> {
pub fn new(content: D, path: Option<PathBuf>) -> Self
pub fn execute<C: Command<D>>(&mut self, cmd: C) -> Result<(), CommandError>
pub fn undo(&mut self) -> Option<&'static str>
pub fn redo(&mut self) -> Option<&'static str>
pub fn is_dirty(&self) -> bool
```

Headless code that does not need undo may mutate the model directly (as the T-shirt example does for building a `Style`), then `validate_*`.
