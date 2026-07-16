use anyhow::{Context, Result};
use clap::Args;
use no_mistakes::cli::{resolve_root, Format};
use no_mistakes::data_pw_query::{self, DataPwInclude, DataPwReport};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Args, Debug)]
pub(crate) struct DataPwArgs {
    /// The selector-attribute value to find (e.g. `search-bar`).
    pub(crate) value: String,
    #[arg(long, default_value = ".", global = true)]
    pub(crate) root: PathBuf,
    #[arg(long, global = true)]
    pub(crate) config: Option<PathBuf>,
    /// Attribute name(s) to scan instead of the configured `testIds`.
    #[arg(long = "attribute", global = true)]
    pub(crate) attributes: Vec<String>,
    /// Source path prefix(es) to scan instead of the configured `selectorRoots`.
    #[arg(long, global = true)]
    pub(crate) scan: Vec<String>,
    /// Sections to include: comma-separated subset of `source,test` (default: all).
    #[arg(long, global = true)]
    pub(crate) include: Option<String>,
    /// Output format: json, yml, md, paths, human.
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
}

pub(crate) fn run(args: DataPwArgs) -> Result<ExitCode> {
    let DataPwArgs {
        value,
        root,
        config,
        attributes,
        scan,
        include,
        format,
        json,
    } = args;
    let effective_format = if json { Format::Json } else { format };
    let cwd = std::env::current_dir().context("cwd must be accessible")?;
    let root = resolve_root(&root, &cwd);
    let include = DataPwInclude::parse(include.as_deref())?;
    let report = data_pw_query::run(
        &root,
        config.as_deref(),
        &value,
        &attributes,
        &scan,
        &include,
    )?;
    no_mistakes::invocation::check_timeout()?;
    match effective_format {
        Format::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&report)
                    .expect("serialization of Rust structs never fails")
            );
        }
        Format::Yml => {
            println!(
                "{}",
                serde_yaml::to_string(&report).expect("serialization of Rust structs never fails")
            );
        }
        Format::Md => print_md(&report),
        Format::Paths => {
            for path in report.paths() {
                println!("{path}");
            }
        }
        Format::Human => print_human(&report),
    }
    Ok(ExitCode::SUCCESS)
}

fn print_human(report: &DataPwReport) {
    println!(
        "{} (attributes: {})",
        report.value,
        report.attributes.join(", ")
    );
    print_section_human("source", report.source.as_deref());
    print_section_human("test", report.test.as_deref());
}

fn print_section_human(label: &str, hits: Option<&[data_pw_query::DataPwHit]>) {
    let Some(hits) = hits else {
        return;
    };
    println!("  {label} ({})", hits.len());
    for hit in hits {
        println!("    {}:{} [{}]", hit.file, hit.line, hit.attribute);
    }
}

fn print_md(report: &DataPwReport) {
    println!("# data-pw `{}`", report.value);
    print_section_md("Source", report.source.as_deref());
    print_section_md("Test", report.test.as_deref());
}

fn print_section_md(label: &str, hits: Option<&[data_pw_query::DataPwHit]>) {
    let Some(hits) = hits else {
        return;
    };
    println!("\n## {label}");
    for hit in hits {
        println!("- `{}:{}` ({})", hit.file, hit.line, hit.attribute);
    }
}
