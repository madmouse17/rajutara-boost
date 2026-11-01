//! Style (`.optx`) -> Marker -> Nesting -> HPGL/2 plot, headless: one Bundle of every size of the Style, nested
//! on a fabric of the given width, saved as `.optm` and plotted as `.plt` next to `<out_stem>`.
//!
//! `cargo run --release -p rajutara --example nest_from_optx -- <style.optx> <out_stem> [seconds=10] [width_mm=1800]`
//!
//! The Marker uses the Material of the first piece that names one, else the Style's default Material.

use std::error::Error;
use std::path::Path;
use std::time::Duration;

use rajutara_core::{
    BundleDirection, Document, InstanceId, Pose, StyleRefId, Unit, validate_marker,
};
use rajutara_io::{PlotterSetup, embed_style, save_marker, utc_now_stamp};
use rajutara_nest::engine::{Control, Stop};
use rajutara_nest::marker::commands::{add_style, apply_nest, set_request, set_setup};
use rajutara_nest::marker::instances::ShapeBook;
use rajutara_nest::marker::metrics::metrics;
use rajutara_nest::marker::props::MarkerSetup;
use rajutara_nest::output::plot::{PlotTarget, header_lines, marker_job, plot_marker};
use rajutara_nest::report::ReportFmt;
use rajutara_nest::{NestMarkerOptions, nest_marker};
use rajutara_ui::{I18n, Lang};

type Res<T> = Result<T, Box<dyn Error>>;

const BUFFER_MM: f64 = 5.0;

fn main() -> Res<()> {
    let mut args = std::env::args().skip(1);
    let (Some(optx), Some(stem)) = (args.next(), args.next()) else {
        return Err("usage: nest_from_optx <style.optx> <out_stem> [seconds] [width_mm]".into());
    };
    let seconds: u64 = args.next().map_or(Ok(10), |s| s.parse())?;
    let width: f64 = args.next().map_or(Ok(1800.0), |s| s.parse())?;

    let (embedded, issues) = embed_style(Path::new(&optx), StyleRefId::from(0))?;
    println!(
        "style {:?}: {} load issue(s)",
        embedded.style.name,
        issues.len()
    );
    let style = &embedded.style;
    let material = style
        .pieces
        .iter()
        .map(|p| p.material.as_str())
        .find(|m| !m.is_empty())
        .unwrap_or(&style.default_material)
        .to_owned();

    let mut setup = MarkerSetup::new(format!("{stem}-marker"), BUFFER_MM);
    setup.width = width;
    let marker = setup
        .into_marker()
        .map_err(|errors| format!("invalid Marker Properties: {errors:?}"))?;
    let mut doc = Document::new(marker, None);
    doc.execute(add_style(embedded.clone()))?;
    let mut setup = MarkerSetup::from_marker(&doc.content);
    setup.material = material;
    doc.execute(set_setup(setup))?;
    let style_ref = doc.content.styles[0].id;
    for size in &style.size_table.sizes {
        doc.execute(set_request(
            style_ref,
            size.name.clone(),
            1,
            BundleDirection::Any,
        ))?;
    }

    let options = NestMarkerOptions {
        stop: Stop::Time(Duration::from_secs(seconds)),
        ..NestMarkerOptions::default()
    };
    let nested = nest_marker(&doc.content, &options, &|_| Control::Continue)?;
    let moves: Vec<(InstanceId, Option<Pose>)> = nested
        .marker
        .pieces
        .iter()
        .map(|p| (p.id, p.pose))
        .collect();
    doc.execute(apply_nest(moves))?;
    let marker = &doc.content;
    println!(
        "nested: efficiency {:.2} %, length {:.3} m, {} unplaced, {} validation issue(s)",
        nested.efficiency_pct,
        nested.length / 1000.0,
        nested.unplaced,
        validate_marker(marker).len()
    );

    let mut book = ShapeBook::default();
    let figures = metrics(marker, &mut book);
    println!(
        "placed {} of {} instances",
        figures.placed, figures.requested
    );

    let optm = format!("{stem}.optm");
    save_marker(marker, Path::new(&optm))?;

    let fmt = ReportFmt {
        unit: Unit::Cm,
        decimals: 2,
        decimal_comma: false,
    };
    let mut i18n = I18n::new(Lang::En);
    i18n.add(rajutara_nest::i18n::catalog());
    let header = header_lines(marker, &mut book, &utc_now_stamp(), &fmt, &|id| {
        i18n.get(id)
    });
    let job = marker_job(marker, &mut book, &header)?;
    let roll = PlotterSetup {
        paper_width: width + 100.0,
        ..rajutara_nest::output::plot::setup_from_json(None)
    };
    let plt = format!("{stem}.plt");
    plot_marker(&job, &PlotTarget::Hpgl(roll), Path::new(&plt))?;
    println!("wrote {optm} and {plt}");
    Ok(())
}
