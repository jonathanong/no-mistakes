use crate::fetches::pipeline::run::run_with_base_root;
use crate::fetches::report::print::write_markdown_report;
use anyhow::{Context, Result};
use clap::Parser;
use no_mistakes::cli::Format;
use std::collections::BTreeSet;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Clone)]
pub(crate) struct FetchesArgs {
    /// Project root directory.
    #[arg(long, default_value = ".", global = true)]
    pub(crate) root: PathBuf,

    /// Config file path. Relative paths are resolved from --root.
    #[arg(long, global = true)]
    pub(crate) config: Option<PathBuf>,

    /// Output format: json, yml, paths, md, human.
    #[arg(
        long,
        value_enum,
        default_value = "human",
        global = true,
        conflicts_with = "json"
    )]
    pub(crate) format: Format,

    /// Shorthand for --format json.
    #[arg(long, global = true, conflicts_with = "format")]
    pub(crate) json: bool,

    #[arg(help = "Specific routes or files to analyze")]
    pub(crate) targets: Vec<String>,
}

pub(crate) type Cli = FetchesArgs;

fn parse_cli_args() -> FetchesArgs {
    if cfg!(test) {
        if let Ok(raw_args) = std::env::var("FETCHES_TEST_ARGS") {
            return FetchesArgs::parse_from(raw_args.split('\u{1f}'));
        }
    }
    FetchesArgs::parse()
}

pub(crate) fn run_cli() -> Result<ExitCode> {
    run(parse_cli_args())
}

const _: fn() -> Result<ExitCode> = run_cli;

pub(crate) fn run(cli: FetchesArgs) -> Result<ExitCode> {
    let base_root = std::env::current_dir().context("reading current directory")?;
    let report = run_with_base_root(&base_root, &cli)?;
    let format = if cli.json { Format::Json } else { cli.format };
    publish_report_with_deadline_check(
        &report,
        format,
        &mut std::io::stdout().lock(),
        no_mistakes::invocation::check_timeout,
    )?;
    Ok(ExitCode::SUCCESS)
}

pub(crate) fn publish_report_with_deadline_check<F>(
    report: &crate::fetches::report::types::FinalReport,
    format: Format,
    writer: &mut dyn Write,
    mut check_deadline: F,
) -> Result<()>
where
    F: FnMut() -> Result<()>,
{
    check_deadline()?;
    let output = render_report(report, format)?;
    check_deadline()?;
    writer.write_all(&output).context("publishing fetch report")
}

fn render_report(
    report: &crate::fetches::report::types::FinalReport,
    format: Format,
) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    match format {
        Format::Json => {
            writeln!(
                output,
                "{}",
                serde_json::to_string_pretty(&report)
                    .expect("serialization of Rust structs never fails")
            )?;
        }
        Format::Yml => writeln!(
            output,
            "{}",
            serde_yaml::to_string(&report).expect("serialization of Rust structs never fails")
        )?,
        Format::Paths => {
            for file in report
                .routes
                .iter()
                .flat_map(|r| {
                    std::iter::once(r.file.as_str())
                        .chain(r.api_calls.iter().map(|f| f.file.as_str()))
                })
                .collect::<BTreeSet<_>>()
            {
                writeln!(output, "{file}")?;
            }
        }
        Format::Md | Format::Human => write_markdown_report(report, &mut output)?,
    }
    Ok(output)
}
