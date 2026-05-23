const DEFAULT_FRONTEND_ROOT: &str = "web/app";
use crate::codebase::dependencies::Format;
use clap::Args;
use is_terminal::IsTerminal;
use std::io;
use std::io::Write;

#[derive(Args, Debug, Clone)]
pub struct CoverageArgs {
    /// Project root directory (default: current working directory).
    #[arg(long, value_name = "PATH")]
    pub root: Option<PathBuf>,

    /// Next.js App Router root. Overrides route-consistency.frontendRoot.
    #[arg(long, value_name = "PATH")]
    pub frontend_root: Option<PathBuf>,

    /// Playwright test file glob. Defaults cover tests/e2e and playwright specs.
    #[arg(long = "test-glob", value_name = "GLOB")]
    pub test_globs: Vec<String>,

    /// Output format: json, md, yml, paths, human.
    /// Defaults to human on TTY, json on non-TTY.
    #[arg(long, value_name = "FORMAT")]
    pub format: Option<Format>,

    /// Shorthand for --format json.
    #[arg(long, default_value_t = false)]
    pub json: bool,

    /// Emit phase timings to stderr.
    #[arg(long, default_value_t = false)]
    pub timings: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitStatus {
    Covered,
    Uncovered,
}

impl ExitStatus {
    pub fn code(self) -> i32 {
        match self {
            Self::Covered => 0,
            Self::Uncovered => 1,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageReport {
    pub summary: CoverageSummary,
    pub routes: Vec<RouteCoverage>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageSummary {
    pub total: usize,
    pub covered: usize,
    pub uncovered: usize,
    pub coverage_percent: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RouteCoverage {
    pub route: String,
    pub file: String,
    pub covered: bool,
    pub tests: Vec<RouteTestHit>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct RouteTestHit {
    pub file: String,
    pub url: String,
}

#[derive(Debug, Clone)]
struct PlaywrightVisit {
    file: PathBuf,
    url: String,
}
