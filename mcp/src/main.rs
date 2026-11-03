//! rajutara MCP server over stdio. Tools: `rajutara_smoke`, `rajutara_import_dxf`, `rajutara_nest`, `rajutara_plot`.

mod tools;

use std::path::Path;

use rmcp::{
    ServiceExt, handler::server::wrapper::Parameters, schemars, tool, tool_router, transport::stdio,
};
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct SmokeParams {
    /// Absolute path of a `.rjx`, `.rjm`, `.rjc` or `.dxf` file (the pre-rename `.optx`, `.optm`, `.optc` too).
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
    /// Absolute path of a Marker (`.rjm`) with instances waiting in the Piece Bar.
    marker: String,
    /// Wall-clock limit of the nesting run, at least 1.
    seconds: u64,
    /// Where to save the nested Marker; omitted: nothing is written.
    out: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct PlotParams {
    /// Absolute path of a nested Marker (`.rjm`).
    marker: String,
    /// `pdf` (A0) or `hpgl` (HPGL/2 for a roll plotter).
    format: String,
    /// Output file; omitted: next to the Marker with extension `.pdf` / `.plt`.
    out: Option<String>,
}

fn reply(result: tools::ToolResult) -> Result<String, String> {
    result.and_then(|v| serde_json::to_string_pretty(&v).map_err(|e| e.to_string()))
}

#[derive(Clone)]
struct Rajutara;

#[tool_router(server_handler)]
impl Rajutara {
    #[tool(
        description = "Load a .rjx/.rjm/.rjc/.dxf file, validate it and summarise it (pieces, sizes, issues)."
    )]
    fn rajutara_smoke(&self, Parameters(p): Parameters<SmokeParams>) -> Result<String, String> {
        reply(tools::smoke(Path::new(&p.path)))
    }

    #[tool(
        description = "Import an ASTM/AAMA/plain DXF (+ RUL grade rules) and return a Style summary with import and validation issues."
    )]
    fn rajutara_import_dxf(
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
    fn rajutara_nest(&self, Parameters(p): Parameters<NestParams>) -> Result<String, String> {
        reply(tools::nest(
            Path::new(&p.marker),
            p.seconds,
            p.out.as_deref().map(Path::new),
        ))
    }

    #[tool(description = "Plot a nested Marker as pdf or hpgl and return the written path.")]
    fn rajutara_plot(&self, Parameters(p): Parameters<PlotParams>) -> Result<String, String> {
        reply(tools::plot(
            Path::new(&p.marker),
            &p.format,
            p.out.as_deref().map(Path::new),
        ))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Rajutara.serve(stdio()).await?.waiting().await?;
    Ok(())
}
