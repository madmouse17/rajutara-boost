# Core model (`rajutara-core`)

Condensed from `docs/spec/01-core-model.md`. Everything is re-exported at the crate root: `use rajutara_core::{Style, Piece, ...}`. Geometry types are `kurbo` (`Point`, `Vec2`, `Rect`, `BezPath`), re-exported.

Unit: mm, `f64`. Ids are typed newtypes (`PieceId`, `PointId`, `NotchId`, ...) handed out by an `IdAlloc` that lives in the document (`style.ids`, `marker.ids`, `plan.ids`): `ids.next()` infers the id type from where it is used. Never invent or reuse an id.

## Style

A `Style` is the unit a `.rjx` file saves. `pieces` order is the Pieces Window, Plot and export order. Pieces are `Arc<Piece>`; mutate with `Arc::make_mut` or through a `Command`.

```rust sig
pub struct Style {
pub name: String,
pub author: String,
pub notes: String,
pub size_table: Arc<SizeTable>,
pub rule_table: Arc<RuleTable>,
pub pieces: Vec<Arc<Piece>>,
pub default_material: String,
pub equal_segments: Vec<EqualGroup>,
pub fabric: Option<FabricPattern>,
pub ids: IdAlloc,
pub fn new(name: impl Into<String>) -> Self
pub fn piece(&self, id: PieceId) -> Option<&Arc<Piece>>
pub fn piece_mut(&mut self, id: PieceId) -> Result<&mut Piece, CommandError>
```

`Style::new` gives one size "M" as Base Size, no pieces, default material "Main". `equal_segments` (Equal Segments Groups: runs kept equal in length, or in shape, while editing) and `fabric` (the stripe or check repeat, `None` = plain) are new in 1.5.0.

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
pub stack: StackPoint,
pub cut: CutRules,
pub placement: Option<Affine>,
pub spread: bool,
pub zones: Vec<Zone>,
pub from_zone: Option<ZoneRef>,
pub color: Option<[u8; 3]>,
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
pub name: String,
pub fn new(id: PointId, at: Point, kind: PointKind) -> Self
pub enum PointKind {
pub struct SeamAllowance {
pub width: f64,
pub end_width: Option<f64>,
pub fn bounding_box(&self) -> Rect
pub fn index_of(&self, id: PointId) -> Option<usize>
pub fn length_between(&self, from: usize, to: usize) -> f64
```

New in 1.5.0, all optional (absent from older files, not written when empty, native format still 2; a 1.4.0 build drops them on re-save and refuses a Style that uses `StackPoint::AlongLine`): `Vertex.name` is the Optitex Point Name (saved, not drawn, not in DXF). `Piece.placement` is Fixed Placement (rigid desk moves since Save Placement), `spread` is Spread Pieces (derived copies drawn beside the piece), `zones` are the Zones of a Map Piece (`ZoneSource::Area` or `Traced`) and `from_zone` the zone a piece was created from, `color` the Work Area display colour (`None` = theme colour). `CutRules.stripe_group` and `stripe_point` are Set Stripes Group and Relate Piece To Stripes. Any field holding ids must be renumbered in `Piece::remap_ids` (paste and merge use it).

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

## Style data added in 1.5.0

```rust sig
pub struct EqualGroup {
pub id: EqualGroupId,
pub name: String,
pub length_only: bool,
pub proportional: bool,
pub color: [u8; 3],
pub members: Vec<EqualMember>,
pub struct EqualMember {
pub piece: PieceId,
pub line: Option<InternalLineId>,
pub from: PointId,
pub to: PointId,
pub ratio: f64,
pub fix_last: bool,
pub struct FabricPattern {
pub stripe: Vec2,
pub gap: Vec2,
pub struct Zone {
pub id: ZoneId,
pub source: ZoneSource,
pub enum ZoneSource {
pub struct ZoneEdge {
pub line: Option<InternalLineId>,
pub struct ZoneRef {
pub map: PieceId,
pub zone: ZoneId,
pub one_line: bool,
pub separator: String,
```

- `EqualGroup`: an Equal Segments Group. `EqualMember.line: None` is a run of the contour; `ratio` is the run's length over the first member's (1.0 unless `proportional`), finite and > 0. Validation refuses a member whose piece or run does not exist.
- `FabricPattern`: `stripe` is one repeat (width x height, > 0), `gap` the space between repeats (>= 0); the grid origin is the desk origin.
- `ZoneSource::Area { seeds }` (Build Zone: a point inside each area) or `Traced { edges }` (Trace Zone: the confining runs, `ZoneEdge.line: None` = contour). `ZoneRef` ties a created piece to its Map Piece and zone.
- `PieceLabel.one_line` prints all label fields on one line joined by `separator` (a space when empty).

The PDS operations that keep this data consistent live in `rajutara-pds` (`ops::equal_segments`, `ops::zones`), not in core. Call them inside a `Command` so the edit is one undo step:

```rust sig
pub fn enforce(before: &Style, after: &mut Style) -> Vec<Msg>
pub fn read_run(piece: &Piece, member: &EqualMember) -> Option<Run>
pub fn add_zone(style: &mut Style, piece: PieceId, name: String, source: ZoneSource) -> Result<ZoneId, CommandError>
pub fn create_pieces_from_map(style: &mut Style, map: PieceId) -> Result<Vec<PieceId>, CommandError>
pub fn correct_pieces_by_map(style: &mut Style) -> Result<usize, CommandError>
pub fn zone_polygon(piece: &Piece, zone: &Zone) -> Option<Vec<Point>>
```

- `equal_segments::enforce(before, after)`: per group, the first member whose run changed is the master and every other member follows it (length only, or shape; proportional, `fix_last`, `mirror`, `inverse`). PDS runs it after every edit through `ops::edit`; a member on a protected piece is skipped and reported in the returned notices. Headless code that edits a Style with groups must call it the same way or the groups drift.
- `zones::add_zone(style, piece, name, source) -> Result<ZoneId, CommandError>` names a zone on a Map Piece (refused when `source` does not enclose one area); `create_pieces_from_map(style, map) -> Result<Vec<PieceId>, CommandError>` makes one piece per zone (all or nothing) with `from_zone` set; `correct_pieces_by_map` rebuilds those pieces after the map changed, keeping their properties, and returns how many.
- `SeamCorner::Custom { depth_in, depth_out }` (Optitex Custom corner, Depth A and Depth B) is enabled since 1.5.0.

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
pub enum StackPoint {
AlongLine {
```

A point in `Piece.grading` (keyed by `PointId`) is a Grade Point. `Piece.stack: StackPoint` (`None`, `Point`, `Grainline`, and since 1.5.0 `AlongLine { from, to }`: the run's arc-length midpoints coincide and its directions are parallel) aligns the Graded Nest for display only. A contour vertex without an entry is graded in proportion to its neighbours; other points are not graded.

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
pub clusters: Vec<Cluster>,
pub cut_order: Vec<CutStep>,
pub bump_lines: Vec<BumpLine>,
pub cut_lines: CutLineSettings,
pub flaws: Vec<Flaw>,
pub texts: Vec<MarkerText>,
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
pub pinned: bool,
pub struct Pose {
pub at: Point,
pub rotation: f64,
pub flip: Flip,
```

`Pose`: the piece is turned so its grainline points along +X, mirrored if `mirrored`, then `rotation` and `flip` about the grainline start, which lands on `at`. `BundleDirection` is `OneWay | Opposite | Any`; `Flip` is `None | X | Y`.

Marker data added in 1.5.0 (`pinned` came in 1.4.0). All optional: a Marker without them writes no new key and the native format stays 2.

```rust sig
pub struct Cluster {
pub id: ClusterId,
pub members: Vec<ClusterMember>,
pub block: Option<FuseBlock>,
pub struct ClusterMember {
pub instance: InstanceId,
pub struct FuseBlock {
pub shape: BlockShape,
pub gap: f64,
pub struct CutStep {
pub start: Option<u32>,
pub direction: CutDirection,
pub struct BumpLine {
pub axis: LineAxis,
pub at: f64,
pub no_bump: bool,
pub struct CutLineSettings {
pub overcut_top: f64,
pub min_segment: f64,
pub struct Flaw {
pub rect: Rect,
pub whole_length: bool,
pub whole_width: bool,
pub fn area(&self, marker: &Marker) -> Rect
pub struct MarkerText {
pub text: String,
pub on: Option<InstanceId>,
```

- `pinned` (Fix on Marker): drags, Shoot, nudges and Auto Nesting never move the instance, even with `keep_placed: false`.
- `Cluster` (Cluster Pieces): members keep their poses relative to the cluster origin and move, nest and return as one; a cluster only translates. With `block` it is a Fuse Block (`BlockShape::Rectangular | Optimum`, `gap` 0 to 1000 mm) whose border Auto Nesting keeps free and the cut file cuts instead of the pieces.
- `CutStep` (Cutting Order): instances in `cut_order` are cut first, in order (a Fuse Block by its first member); `start` is the segment-start index of the knife (`None` = lowest-left vertex), `direction` defaults to `CutDirection::Clockwise`. The others follow by X, then Y.
- `Flaw`: fabric no piece may overlap; `area` resolves `whole_length` / `whole_width` to the marker length and material width. `BumpLine`: a vertical or horizontal wall (`cut` = also a Cut Line, `no_bump` = pieces may cross). `MarkerText.on: Some(instance)` moves the text with that piece.

## CutPlan

The Lays that cut one `Order` from one `Material`. Behaviour is in `rajutara-cutplan` (`cutplan.md`).

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

Validate before you trust a document and refuse on `Severity::Error`; warnings ride along. Loaders in `rajutara-io` already do this.

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
