//! optirust MCP server over stdio. Tools: `optirust_smoke`, `optirust_import_dxf`, `optirust_nest`, `optirust_plot`.

mod tools;

use std::path::Path;

use rmcp::{
    ServiceExt, handler::server::wrapper::Parameters, schemars, tool, tool_router, transport::stdio,
};
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct SmokeParams {
    /// Absolute path of a `.optx`, `.optm`, `.optc` or `.dxf` file.
    path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ImportParams {
    /// Absolute path of the DXF file.
    path: String,
    /// Absolute path of an ASTM `.rul` file; omitted: the importer looks for the `.rul` the DXF names, next to it.
    rul: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct NestParams {
    /// Absolute path of a Marker (`.optm`) with instances waiting in the Piece Bar.
    optm: String,
    /// Wall-clock limit of the nesting run, at least 1.
    seconds: u64,
    /// Where to save the nested Marker; omitted: nothing is written.
    out: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct PlotParams {
    /// Absolute path of a nested Marker (`.optm`).
    optm: String,
    /// `pdf` (A0) or `hpgl` (HPGL/2 for a roll plotter).
    format: String,
    /// Output file; omitted: next to the Marker with extension `.pdf` / `.plt`.
    out: Option<String>,
}

fn reply(result: tools::ToolResult) -> Result<String, String> {
    result.and_then(|v| serde_json::to_string_pretty(&v).map_err(|e| e.to_string()))
}

#[derive(Clone)]
struct Optirust;

#[tool_router(server_handler)]
impl Optirust {
    #[tool(
        description = "Load a .optx/.optm/.optc/.dxf file, validate it and summarise it (pieces, sizes, issues)."
    )]
    fn optirust_smoke(&self, Parameters(p): Parameters<SmokeParams>) -> Result<String, String> {
        reply(tools::smoke(Path::new(&p.path)))
    }

    #[tool(
        description = "Import an ASTM/AAMA/plain DXF (+ RUL grade rules) and return a Style summary with import and validation issues."
    )]
    fn optirust_import_dxf(
        &self,
        Parameters(p): Parameters<ImportParams>,
    ) -> Result<String, String> {
        reply(tools::import(
            Path::new(&p.path),
            p.rul.as_deref().map(Path::new),
        ))
    }

    #[tool(
        description = "Nest a Marker for `seconds` and return efficiency, length, unplaced count and overlaps."
    )]
    fn optirust_nest(&self, Parameters(p): Parameters<NestParams>) -> Result<String, String> {
        reply(tools::nest(
            Path::new(&p.optm),
            p.seconds,
            p.out.as_deref().map(Path::new),
        ))
    }

    #[tool(description = "Plot a nested Marker as pdf or hpgl and return the written path.")]
    fn optirust_plot(&self, Parameters(p): Parameters<PlotParams>) -> Result<String, String> {
        reply(tools::plot(
            Path::new(&p.optm),
            &p.format,
            p.out.as_deref().map(Path::new),
        ))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Optirust.serve(stdio()).await?.waiting().await?;
    Ok(())
}
