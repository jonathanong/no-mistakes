use super::render::{render, resolve_format, to_json, Report};
use super::reverse::build_index;
use super::shared::{rel_str, resolve_target, Target};
use crate::cli::Format;
use crate::tests::impact::generate_impact_plan;
use crate::tests::ImpactArgs;
use anyhow::Result;
use is_terminal::IsTerminal;
use serde::Serialize;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

/// `importers`: the files that directly import a file, with a dependents count.
/// `--tests` adds the transitive impacted-test set (builds the dependency graph).
#[derive(clap::Parser, Debug)]
pub struct ImportersArgs {
    /// The TS/JS file whose importers to find (relative to --root or absolute).
    #[arg(value_name = "FILE")]
    pub file: PathBuf,

    /// Also compute the transitive set of tests impacted by this file.
    #[arg(long, default_value_t = false)]
    pub tests: bool,

    /// Project root (default: current working directory).
    #[arg(long, value_name = "PATH")]
    pub root: Option<PathBuf>,

    /// Path to tsconfig.json for resolving import specifiers.
    #[arg(long, value_name = "FILE")]
    pub tsconfig: Option<PathBuf>,

    /// Output format: json, yml, md, paths, human.
    #[arg(long, value_name = "FORMAT")]
    pub format: Option<Format>,

    /// Shorthand for `--format json`.
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TestImpact {
    tests: Vec<String>,
    count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportersReport {
    file: String,
    direct_importers: Vec<String>,
    dependents_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    test_impact: Option<TestImpact>,
}

fn test_impact(args: &ImportersArgs, target: &Target) -> Result<TestImpact> {
    let plan = generate_impact_plan(&ImpactArgs {
        entrypoints: vec![target.abs_file.display().to_string()],
        // Pass an empty structured symbol so a literal `#` in the path is not
        // parsed as a `file#symbol` entrypoint.
        entrypoint_symbols: vec![Some(String::new())],
        include_symbols: false,
        root: target.root.clone(),
        config: None,
        tsconfig: args.tsconfig.clone(),
        format: None,
        json: false,
    })?;
    let tests: Vec<String> = plan
        .selected_tests
        .into_iter()
        .map(|test| test.test_file)
        .collect();
    Ok(TestImpact {
        count: tests.len(),
        tests,
    })
}

fn compute(args: &ImportersArgs) -> Result<ImportersReport> {
    let target = resolve_target(&args.file, args.root.as_deref(), args.tsconfig.as_deref())?;
    let index = build_index(&target)?;
    let direct_importers: Vec<String> = index
        .file_importers(&target.abs_file)
        .iter()
        .map(|importer| rel_str(importer, &target.root))
        .collect();

    let test_impact = if args.tests {
        Some(test_impact(args, &target)?)
    } else {
        None
    };

    Ok(ImportersReport {
        file: rel_str(&target.abs_file, &target.root),
        dependents_count: direct_importers.len(),
        direct_importers,
        test_impact,
    })
}

impl Report for ImportersReport {
    fn write_human(&self, w: &mut dyn Write) -> io::Result<()> {
        writeln!(w, "{} ({} dependents)", self.file, self.dependents_count)?;
        for importer in &self.direct_importers {
            writeln!(w, "  {importer}")?;
        }
        if let Some(impact) = &self.test_impact {
            writeln!(w, "impacts {} tests:", impact.count)?;
            for test in &impact.tests {
                writeln!(w, "  {test}")?;
            }
        }
        Ok(())
    }

    fn write_paths(&self, w: &mut dyn Write) -> io::Result<()> {
        for importer in &self.direct_importers {
            writeln!(w, "{importer}")?;
        }
        Ok(())
    }
}

pub fn run(args: ImportersArgs) -> Result<ExitCode> {
    let report = compute(&args)?;
    let format = resolve_format(args.json, args.format, io::stdout().is_terminal());
    let stdout = io::stdout();
    let mut out = stdout.lock();
    render(&report, format, &mut out)?;
    Ok(ExitCode::SUCCESS)
}

pub fn run_json(args: ImportersArgs) -> Result<String> {
    to_json(&compute(&args)?)
}

#[cfg(test)]
mod tests;
