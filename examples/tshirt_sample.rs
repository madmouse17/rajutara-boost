//! Writes a complete short-sleeve T-shirt sample ("Kaos Lengan Pendek") headless: a Style drafted from the
//! public basic tee block, graded S / M / L / XL, a one-Bundle-per-size Marker nested on 1800 mm Single
//! Jersey, and every output of the workflow next to it.
//!
//! `cargo run --release -p optirust --example tshirt_sample -- <out_dir>`
//!
//! The block is plain pattern-making arithmetic for Base Size M (chest 100 cm, body length 70 cm, underarm
//! sleeve seam 22 cm, neck width 18 cm). Pieces are full (no Mirror Line) so the Marker is realistic; the
//! Contour is the sewing line and every vertex carries a 1 cm Seam Allowance.

use std::error::Error;
use std::fs;
use std::path::Path;
use std::time::Duration;

use optirust_core::grading::piece_for_size;
use optirust_core::{
    Annotation, BundleDirection, Contour, Document, EmbeddedStyle, GradeRule, IdAlloc, InstanceId,
    Issue, Marker, Notch, NotchAt, NotchPlace, NotchRelation, NotchShape, NumberedRule, Piece,
    PiecePoint, Point, PointKind, RuleTable, SeamAllowance, SegmentShape, Severity, Size,
    SizeTable, Style, StyleRefId, Unit, Vec2, Vertex, validate_marker, validate_style,
};
use optirust_grading::export::{ExportRequest, run_export};
use optirust_io::{
    GradingExport, PdfPage, embed_style, load_marker, load_style, plot_pdf, save_marker,
    save_style, utc_now_stamp,
};
use optirust_nest::engine::{Control, Stop};
use optirust_nest::marker::commands::{add_style, apply_nest, set_request, set_setup};
use optirust_nest::marker::instances::ShapeBook;
use optirust_nest::marker::metrics::{conflicts, metrics};
use optirust_nest::marker::props::MarkerSetup;
use optirust_nest::output::plot::{
    PlotTarget, header_lines, marker_job, plot_marker, setup_from_json,
};
use optirust_nest::report::{ReportFmt, ReportMeta, report_data, write_csv_report};
use optirust_nest::{NestMarkerOptions, nest_marker};
use optirust_pds::plot::job::plot_job;
use optirust_pds::tools::{FieldValue, Values};
use optirust_ui::{I18n, Lang, Selection};
use std::sync::Arc;

type Res<T> = Result<T, Box<dyn Error>>;

const STEM: &str = "kaos-lengan-pendek";
const SIZES: [&str; 4] = ["S", "M", "L", "XL"];
const BASE: usize = 1;
const MATERIAL: &str = "Single Jersey";
const FABRIC_WIDTH_MM: f64 = 1800.0;
const BUFFER_MM: f64 = 5.0;
const NEST_SECONDS: u64 = 15;

// Base Size M, millimetres, X right, Y up, hem of the body at Y = 0.
const SEAM_MM: f64 = 10.0;
const HALF_CHEST: f64 = 250.0; // chest 100 cm, front and back each carry half
const BODY_LENGTH: f64 = 700.0; // high point shoulder to hem
const UNDERARM_Y: f64 = 460.0;
const SHOULDER_X: f64 = 211.0;
const SHOULDER_Y: f64 = 650.0; // shoulder drops 50 mm from the neck point
const NECK_HALF: f64 = 90.0; // neck width 18 cm
const FRONT_NECK_DEPTH: f64 = 80.0;
const BACK_NECK_DEPTH: f64 = 20.0;
const FRONT_ARMHOLE: [(f64, f64); 4] = [
    (228.0, 480.0),
    (209.0, 515.0),
    (202.0, 562.0),
    (204.0, 612.0),
];
const BACK_ARMHOLE: [(f64, f64); 4] = [
    (230.0, 478.0),
    (214.0, 512.0),
    (208.0, 560.0),
    (209.0, 610.0),
];
const SLEEVE_SEAM: f64 = 220.0; // sleeve length 22 cm: hem to underarm
const SLEEVE_BICEP_HALF: f64 = 175.0;
const SLEEVE_HEM_HALF: f64 = 160.0;
const SLEEVE_APEX_Y: f64 = SLEEVE_SEAM + 120.0; // cap height 12 cm
const SLEEVE_CAP_BACK: [(f64, f64); 4] =
    [(158.0, 236.0), (125.0, 266.0), (80.0, 306.0), (38.0, 331.0)];
const SLEEVE_CAP_FRONT: [(f64, f64); 4] =
    [(157.0, 233.0), (122.0, 262.0), (78.0, 304.0), (36.0, 331.0)];
const RIB_STRETCH: f64 = 0.85; // rib is cut 15 % shorter than the neck opening
const RIB_HEIGHT: f64 = 40.0;

/// Grade Rules: number, name, step per size away from M (mm). Chest ±4 cm per size is ±1 cm on each side of
/// a full piece, length ±1.5 cm, sleeve length ±0.5 cm, neck ±0.5 cm (±2.5 mm per neck point).
const RULES: [(i32, &str, (f64, f64)); 14] = [
    (1, "Hem right", (10.0, -15.0)),
    (2, "Hem left", (-10.0, -15.0)),
    (3, "Underarm right", (10.0, 0.0)),
    (4, "Underarm left", (-10.0, 0.0)),
    (5, "Shoulder tip right", (5.0, 0.0)),
    (6, "Shoulder tip left", (-5.0, 0.0)),
    (7, "Neck point right", (2.5, 0.0)),
    (8, "Neck point left", (-2.5, 0.0)),
    (9, "Fixed", (0.0, 0.0)),
    (10, "Sleeve hem right", (5.0, -5.0)),
    (11, "Sleeve hem left", (-5.0, -5.0)),
    (12, "Sleeve underarm right", (5.0, 0.0)),
    (13, "Sleeve underarm left", (-5.0, 0.0)),
    (14, "Rib end", (10.0, 0.0)),
];

type Pt = (f64, f64, PointKind);

fn turn(x: f64, y: f64) -> Pt {
    (x, y, PointKind::Turn)
}

fn curve(x: f64, y: f64) -> Pt {
    (x, y, PointKind::Curve)
}

fn tenth(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

/// The deltas of a rule in Size Table order: `step` times the distance from M, zero at M.
fn rule(step: (f64, f64)) -> GradeRule {
    GradeRule {
        deltas: (0..SIZES.len())
            .map(|i| {
                if i == BASE {
                    Vec2::ZERO
                } else {
                    let k = i as f64 - BASE as f64;
                    Vec2::new(k * step.0, k * step.1)
                }
            })
            .collect(),
    }
}

fn rule_numbered(number: i32) -> GradeRule {
    RULES
        .iter()
        .find(|r| r.0 == number)
        .map_or_else(|| GradeRule::zero(SIZES.len()), |r| rule(r.2))
}

/// A Piece on the Sewing line: `pts` as vertices with the 1 cm Seam Allowance, in the one Material.
fn piece(ids: &mut IdAlloc, name: &str, pts: &[Pt]) -> Piece {
    let points = pts
        .iter()
        .map(|&(x, y, kind)| {
            let mut v = Vertex::new(ids.next(), Point::new(x, y), kind);
            v.seam = Some(SeamAllowance {
                width: SEAM_MM,
                end_width: None,
            });
            v
        })
        .collect();
    let contour = Contour {
        points,
        shapes: vec![SegmentShape::FromPoints; pts.len()],
    };
    let mut piece = Piece::new(ids, name, contour);
    piece.material = MATERIAL.to_owned();
    piece
}

fn grade(piece: &mut Piece, vertex: usize, number: i32) {
    piece
        .grading
        .insert(piece.contour.points[vertex].id, rule_numbered(number));
}

fn notch(ids: &mut IdAlloc, piece: &mut Piece, shape: NotchShape, at: NotchAt) {
    piece.notches.push(Notch {
        shape,
        ..Notch::new(ids.next(), at)
    });
}

fn on_vertex(piece: &Piece, vertex: usize) -> NotchAt {
    NotchAt::Point(piece.contour.points[vertex].id)
}

/// Halfway (or `ratio` of the way) along the contour from vertex `from` to vertex `to`, in every size.
fn along(piece: &Piece, from: usize, to: usize, ratio: f64) -> NotchAt {
    NotchAt::Between {
        prev: piece.contour.points[from].id,
        next: piece.contour.points[to].id,
        place: NotchPlace {
            relation: NotchRelation::Proportional,
            value: ratio,
            per_size: Vec::new(),
        },
    }
}

fn grainline(piece: &mut Piece, from: (f64, f64), to: (f64, f64), angle: f64) {
    piece.grainline.start.at = Point::new(from.0, from.1);
    piece.grainline.end.at = Point::new(to.0, to.1);
    piece.grainline.initial_angle = angle;
}

fn annotate(ids: &mut IdAlloc, piece: &mut Piece, at: (f64, f64), text: &str, height: f64) {
    piece.annotations.push(Annotation {
        id: ids.next(),
        at: PiecePoint {
            id: ids.next(),
            at: Point::new(at.0, at.1),
        },
        text: text.to_owned(),
        height,
        angle: 0.0,
    });
}

/// Quarter ellipse from the neck point to the centre, as three Curve Points (22.5°, 45°, 67.5°).
fn neck_side(depth: f64) -> [(f64, f64); 3] {
    [22.5_f64, 45.0, 67.5].map(|deg| {
        let r = deg.to_radians();
        (
            tenth(NECK_HALF * r.cos()),
            tenth(BODY_LENGTH - depth * r.sin()),
        )
    })
}

/// Front or back of the body: full piece, hem left to hem right, up the right side, over the neck and
/// down the left side. Vertex order is fixed (see the constants in the body of the function).
fn body(
    ids: &mut IdAlloc,
    name: &str,
    neck_depth: f64,
    armhole: [(f64, f64); 4],
    armhole_notch: NotchShape,
) -> Piece {
    // 0 hem L, 1 hem R, 2 underarm R, 3..=6 armhole R, 7 shoulder tip R, 8 neck point R, 9..=11 neck R,
    // 12 centre, 13..=15 neck L, 16 neck point L, 17 shoulder tip L, 18..=21 armhole L, 22 underarm L.
    const UNDER_R: usize = 2;
    const TIP_R: usize = 7;
    const NECK_R: usize = 8;
    const CENTRE: usize = 12;
    const NECK_L: usize = 16;
    const TIP_L: usize = 17;
    const UNDER_L: usize = 22;

    let neck = neck_side(neck_depth);
    let mut pts = vec![
        turn(-HALF_CHEST, 0.0),
        turn(HALF_CHEST, 0.0),
        turn(HALF_CHEST, UNDERARM_Y),
    ];
    pts.extend(armhole.iter().map(|&(x, y)| curve(x, y)));
    pts.push(turn(SHOULDER_X, SHOULDER_Y));
    pts.push(turn(NECK_HALF, BODY_LENGTH));
    pts.extend(neck.iter().map(|&(x, y)| curve(x, y)));
    pts.push(curve(0.0, BODY_LENGTH - neck_depth));
    pts.extend(neck.iter().rev().map(|&(x, y)| curve(-x, y)));
    pts.push(turn(-NECK_HALF, BODY_LENGTH));
    pts.push(turn(-SHOULDER_X, SHOULDER_Y));
    pts.extend(armhole.iter().rev().map(|&(x, y)| curve(-x, y)));
    pts.push(turn(-HALF_CHEST, UNDERARM_Y));

    let mut p = piece(ids, name, &pts);
    for (vertex, number) in [
        (0, 2),
        (1, 1),
        (UNDER_R, 3),
        (TIP_R, 5),
        (NECK_R, 7),
        (CENTRE, 9),
        (NECK_L, 8),
        (TIP_L, 6),
        (UNDER_L, 4),
    ] {
        grade(&mut p, vertex, number);
    }
    for (shape, at) in [
        (armhole_notch, along(&p, UNDER_R, TIP_R, 0.5)),
        (armhole_notch, along(&p, TIP_L, UNDER_L, 0.5)),
        (NotchShape::V, along(&p, TIP_R, NECK_R, 0.5)),
        (NotchShape::V, along(&p, NECK_L, TIP_L, 0.5)),
        (NotchShape::V, on_vertex(&p, CENTRE)),
    ] {
        notch(ids, &mut p, shape, at);
    }
    grainline(
        &mut p,
        (0.0, 120.0),
        (0.0, BODY_LENGTH - neck_depth - 40.0),
        std::f64::consts::FRAC_PI_2,
    );
    annotate(ids, &mut p, (30.0, 200.0), &format!("{name}\nCut 1"), 8.0);
    p
}

/// Short sleeve, grain up the middle: hem left, hem right, up the back side over the cap, down the front side.
fn sleeve(ids: &mut IdAlloc) -> Piece {
    // 0 hem L, 1 hem R, 2 underarm R, 3..=6 cap (back), 7 apex, 8..=11 cap (front), 12 underarm L.
    let mut pts = vec![
        turn(-SLEEVE_HEM_HALF, 0.0),
        turn(SLEEVE_HEM_HALF, 0.0),
        turn(SLEEVE_BICEP_HALF, SLEEVE_SEAM),
    ];
    pts.extend(SLEEVE_CAP_BACK.iter().map(|&(x, y)| curve(x, y)));
    pts.push(curve(0.0, SLEEVE_APEX_Y));
    pts.extend(SLEEVE_CAP_FRONT.iter().rev().map(|&(x, y)| curve(-x, y)));
    pts.push(turn(-SLEEVE_BICEP_HALF, SLEEVE_SEAM));

    let mut p = piece(ids, "SLEEVE", &pts);
    for (vertex, number) in [(0, 11), (1, 10), (2, 12), (7, 9), (12, 13)] {
        grade(&mut p, vertex, number);
    }
    for (shape, at) in [
        (NotchShape::V, on_vertex(&p, 7)),
        (NotchShape::T, on_vertex(&p, 4)),
        (NotchShape::I, on_vertex(&p, 10)),
    ] {
        notch(ids, &mut p, shape, at);
    }
    // One sleeve is cut as a left/right pair: the cap is not symmetric.
    p.pair = true;
    grainline(
        &mut p,
        (0.0, 40.0),
        (0.0, 300.0),
        std::f64::consts::FRAC_PI_2,
    );
    annotate(ids, &mut p, (20.0, 120.0), "SLEEVE\nCut 2 (L/R)", 8.0);
    p
}

/// Neck binding: a rectangle `length` long, grain along its length.
fn neck_rib(ids: &mut IdAlloc, length: f64) -> Piece {
    let pts = [
        turn(0.0, 0.0),
        turn(length, 0.0),
        turn(length, RIB_HEIGHT),
        turn(0.0, RIB_HEIGHT),
    ];
    let mut p = piece(ids, "NECK RIB", &pts);
    for (vertex, number) in [(0, 9), (1, 14), (2, 14), (3, 9)] {
        grade(&mut p, vertex, number);
    }
    for ratio in [0.25, 0.5, 0.75] {
        let at = along(&p, 0, 1, ratio);
        notch(ids, &mut p, NotchShape::I, at);
    }
    grainline(
        &mut p,
        (15.0, RIB_HEIGHT / 2.0),
        (length - 15.0, RIB_HEIGHT / 2.0),
        0.0,
    );
    annotate(ids, &mut p, (10.0, 6.0), "Cut 1", 6.0);
    p
}

/// Lengths worth printing: what the draft measures on itself.
struct Measures {
    armhole_front: f64,
    armhole_back: f64,
    cap: f64,
    neck: f64,
    rib: f64,
}

fn tee_style() -> (Style, Measures) {
    let mut style = Style::new("Kaos Lengan Pendek");
    style.author = "optirust sample".to_owned();
    style.notes = "Basic short-sleeve tee block, Base Size M: chest 100 cm, body length 70 cm, \
                   sleeve seam 22 cm, neck width 18 cm. Seam Allowance 1 cm all round."
        .to_owned();
    style.default_material = MATERIAL.to_owned();
    style.size_table = Arc::new(SizeTable {
        sizes: SIZES.map(Size::new).to_vec(),
        base: BASE,
    });
    style.rule_table = Arc::new(RuleTable {
        name: "KAOS-LP".to_owned(),
        rules: RULES
            .iter()
            .map(|&(number, name, step)| NumberedRule {
                number,
                name: name.to_owned(),
                rule: rule(step),
            })
            .collect(),
    });

    let ids = &mut style.ids;
    let front = body(ids, "FRONT", FRONT_NECK_DEPTH, FRONT_ARMHOLE, NotchShape::I);
    let back = body(ids, "BACK", BACK_NECK_DEPTH, BACK_ARMHOLE, NotchShape::T);
    let sleeve = sleeve(ids);
    let neck = front.contour.length_between(8, 16) + back.contour.length_between(8, 16);
    let rib = tenth(neck * RIB_STRETCH);
    let rib_piece = neck_rib(ids, rib);
    let measures = Measures {
        armhole_front: front.contour.length_between(2, 7),
        armhole_back: back.contour.length_between(2, 7),
        cap: sleeve.contour.length_between(2, 12),
        neck,
        rib,
    };
    style.pieces = [front, back, sleeve, rib_piece]
        .into_iter()
        .map(Arc::new)
        .collect();
    (style, measures)
}

fn refuse_on_errors(what: &str, issues: &[Issue]) -> Res<()> {
    let bad: Vec<&Issue> = issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .collect();
    if bad.is_empty() {
        println!(
            "{what}: valid, {} warning(s)",
            issues.len().saturating_sub(bad.len())
        );
        Ok(())
    } else {
        Err(format!(
            "{what} has {} error issue(s), nothing more written: {bad:?}",
            bad.len()
        )
        .into())
    }
}

fn ensure(ok: bool, what: impl FnOnce() -> String) -> Res<()> {
    if ok { Ok(()) } else { Err(what().into()) }
}

/// Grading check: L is 2 cm wider than M on the chest, and every size follows ±2 cm / ±1.5 cm / ±0.5 cm.
fn check_grading(style: &Style) -> Res<()> {
    let table = &style.size_table;
    let bbox = |p: &Piece, size: usize| piece_for_size(p, table, size).contour.bounding_box();
    // (piece, width step, height step) per size away from M
    let steps = [
        (0, 20.0, 15.0),
        (1, 20.0, 15.0),
        (2, 10.0, 5.0),
        (3, 10.0, 0.0),
    ];
    for (index, width_step, height_step) in steps {
        let piece = &style.pieces[index];
        let base = bbox(piece, BASE);
        for (size, name) in SIZES.iter().enumerate() {
            let k = size as f64 - BASE as f64;
            let b = bbox(piece, size);
            ensure(
                (b.width() - (base.width() + k * width_step)).abs() < 0.01
                    && (b.height() - (base.height() + k * height_step)).abs() < 0.01,
                || {
                    format!(
                        "{} size {}: {:.2} x {:.2} mm, expected {:.2} x {:.2}",
                        piece.name,
                        name,
                        b.width(),
                        b.height(),
                        base.width() + k * width_step,
                        base.height() + k * height_step
                    )
                },
            )?;
        }
    }
    let front = &style.pieces[0];
    let (m, l) = (bbox(front, 1).width(), bbox(front, 2).width());
    ensure((l - m - 20.0).abs() < 0.01, || {
        format!("L front is {l:.2} mm wide, M {m:.2}: not 20 mm apart")
    })?;
    println!(
        "grading: front width S/M/L/XL = {:.0}/{m:.0}/{l:.0}/{:.0} mm (L is {:.1} cm wider than M)",
        bbox(front, 0).width(),
        bbox(front, 3).width(),
        (l - m) / 10.0
    );
    Ok(())
}

fn marker_of(embedded: &EmbeddedStyle) -> Res<Document<Marker>> {
    let mut setup = MarkerSetup::new(format!("{STEM}-marker"), BUFFER_MM);
    setup.width = FABRIC_WIDTH_MM;
    let marker = setup
        .into_marker()
        .map_err(|errors| format!("invalid Marker Properties: {errors:?}"))?;
    let mut doc = Document::new(marker, None);
    doc.execute(add_style(embedded.clone()))?;
    let mut setup = MarkerSetup::from_marker(&doc.content);
    setup.material = MATERIAL.to_owned();
    doc.execute(set_setup(setup))?;
    let style = doc.content.styles[0].id;
    for size in SIZES {
        doc.execute(set_request(style, size.to_owned(), 1, BundleDirection::Any))?;
    }
    Ok(doc)
}

fn pieces_values() -> Values {
    let flag = |key, on| (key, FieldValue::Flag(on));
    Values(vec![
        ("sizes", FieldValue::Choice(0)),
        ("target", FieldValue::Choice(0)),
        flag("arrange_first", true),
        ("width", FieldValue::Length(841.0)),
        ("gap", FieldValue::Length(10.0)),
        flag("cut_line", true),
        flag("sew_line", true),
        flag("internals", true),
        flag("notches", true),
        flag("drill_holes", true),
        flag("grainline", true),
        flag("mirror_line", true),
        flag("annotations", true),
        flag("piece_label", true),
    ])
}

fn main() -> Res<()> {
    let out = std::env::args()
        .nth(1)
        .ok_or("usage: tshirt_sample <out_dir>")?;
    let out = Path::new(&out);

    // 1-2. Style, graded and checked before anything is written
    let (style, measures) = tee_style();
    refuse_on_errors("style", &validate_style(&style))?;
    check_grading(&style)?;
    println!(
        "draft: armhole front {:.1} / back {:.1} mm, sleeve cap {:.1} mm (ease {:+.1}), neck {:.1} mm, rib {:.1} mm",
        measures.armhole_front,
        measures.armhole_back,
        measures.cap,
        measures.cap - measures.armhole_front - measures.armhole_back,
        measures.neck,
        measures.rib
    );
    fs::create_dir_all(out)?;
    let optx = out.join(format!("{STEM}.optx"));
    save_style(&style, &optx)?;
    let (embedded, issues) = embed_style(&optx, StyleRefId::from(0))?;
    ensure(issues.is_empty(), || format!("{issues:?}"))?;

    // 3. Marker: one Bundle per size, nested
    let mut doc = marker_of(&embedded)?;
    refuse_on_errors("marker", &validate_marker(&doc.content))?;
    let options = NestMarkerOptions {
        stop: Stop::Time(Duration::from_secs(NEST_SECONDS)),
        seed: 1,
        ..NestMarkerOptions::default()
    };
    let nested = nest_marker(&doc.content, &options, &|_| Control::Continue)?;
    ensure(nested.unplaced == 0, || {
        format!("{} instance(s) did not fit the fabric", nested.unplaced)
    })?;
    let moves: Vec<(InstanceId, Option<optirust_core::Pose>)> = nested
        .marker
        .pieces
        .iter()
        .map(|p| (p.id, p.pose))
        .collect();
    doc.execute(apply_nest(moves))?;
    let marker = &doc.content;
    refuse_on_errors("nested marker", &validate_marker(marker))?;

    let mut book = ShapeBook::default();
    let figures = metrics(marker, &mut book);
    let found = conflicts(marker, &mut book);
    ensure(
        found.overlap_count() == 0 && found.in_margin.is_empty(),
        || format!("conflicts: {found:?}"),
    )?;
    println!(
        "marker: {} pieces placed of {} ({} bundles), efficiency {:.2} %, length {:.3} m, {} conflicts",
        figures.placed,
        figures.requested,
        figures.bundles_placed,
        figures.efficiency_pct,
        figures.used_length / 1000.0,
        found.overlap_count()
    );

    // 4. Outputs
    let optm = out.join(format!("{STEM}-marker.optm"));
    save_marker(marker, &optm)?;

    let date = utc_now_stamp();
    let fmt = ReportFmt {
        unit: Unit::Cm,
        decimals: 2,
        decimal_comma: false,
    };
    let mut i18n = I18n::new(Lang::En);
    i18n.add(optirust_nest::i18n::catalog());
    let tr = |id: &str| i18n.get(id);
    let header = header_lines(marker, &mut book, &date, &fmt, &tr);
    let job = marker_job(marker, &mut book, &header)?;
    plot_marker(
        &job,
        &PlotTarget::Pdf(PdfPage::A0),
        &out.join(format!("{STEM}-marker.pdf")),
    )?;
    let roll = optirust_io::PlotterSetup {
        paper_width: FABRIC_WIDTH_MM + 100.0,
        ..setup_from_json(None)
    };
    plot_marker(
        &job,
        &PlotTarget::Hpgl(roll),
        &out.join(format!("{STEM}-marker.plt")),
    )?;

    let meta = ReportMeta {
        file_name: format!("{STEM}-marker.optm"),
        date: date.clone(),
        fmt,
    };
    let data = report_data(marker, &mut book, &meta, &tr);
    write_csv_report(&data, &out.join(format!("{STEM}-marker-report.csv")))?;

    let (pieces_job, _) = plot_job(&style, &Selection::default(), &pieces_values())?;
    plot_pdf(
        &pieces_job,
        PdfPage::A4Tiled,
        &out.join(format!("{STEM}-pieces-M.pdf")),
    )?;

    run_export(
        &style,
        &ExportRequest {
            path: out.join(format!("{STEM}.dxf")),
            grading: GradingExport::RuleFile,
            visible_sizes_only: false,
            positive_rule_numbers: false,
            metric: true,
            pieces: None,
        },
        date,
    )?;

    // 5. Everything written loads back
    let (loaded_style, style_issues) = load_style(&optx)?;
    let (loaded_marker, marker_issues) = load_marker(&optm)?;
    ensure(loaded_style == style && loaded_marker == *marker, || {
        "a saved file does not load back equal".to_owned()
    })?;
    println!(
        "reload: {STEM}.optx ({} pieces, {} issues) and {STEM}-marker.optm ({} instances, {} issues) load equal",
        loaded_style.pieces.len(),
        style_issues.len(),
        loaded_marker.pieces.len(),
        marker_issues.len()
    );
    Ok(())
}
