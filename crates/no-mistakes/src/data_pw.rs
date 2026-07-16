use anyhow::{Context, Result};
use clap::Args;
use no_mistakes::cli::{resolve_root, Format};
use no_mistakes::data_pw_query::{self, DataPwInclude, DataPwReport};
use std::io::{self, Write};
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
    let output = render_report(&report, effective_format)?;
    no_mistakes::invocation::check_timeout()?;
    std::io::stdout()
        .lock()
        .write_all(&output)
        .context("publishing data-pw output")?;
    Ok(ExitCode::SUCCESS)
}

fn render_report(report: &DataPwReport, format: Format) -> Result<Vec<u8>> {
    render_report_with_deadline_check(report, format, no_mistakes::invocation::check_timeout)
}

fn render_report_with_deadline_check<F>(
    report: &DataPwReport,
    format: Format,
    mut check_deadline: F,
) -> Result<Vec<u8>>
where
    F: FnMut() -> Result<()>,
{
    check_deadline()?;
    let mut output = Vec::new();
    match format {
        Format::Json => {
            let serialized = serde_json::to_string_pretty(&report)
                .expect("serialization of Rust structs never fails");
            output.extend_from_slice(serialized.as_bytes());
            output.push(b'\n');
        }
        Format::Yml => {
            let serialized =
                serde_yaml::to_string(&report).expect("serialization of Rust structs never fails");
            output.extend_from_slice(serialized.as_bytes());
            output.push(b'\n');
        }
        Format::Md => write_md(report, &mut output)?,
        Format::Paths => {
            for path in report.paths() {
                writeln!(output, "{path}")?;
            }
        }
        Format::Human => write_human(report, &mut output)?,
    }
    check_deadline()?;
    Ok(output)
}

fn write_human(report: &DataPwReport, output: &mut dyn Write) -> io::Result<()> {
    writeln!(
        output,
        "{} (attributes: {})",
        report.value,
        report.attributes.join(", ")
    )?;
    write_section_human(output, "source", report.source.as_deref())?;
    write_section_human(output, "test", report.test.as_deref())
}

fn write_section_human(
    output: &mut dyn Write,
    label: &str,
    hits: Option<&[data_pw_query::DataPwHit]>,
) -> io::Result<()> {
    let Some(hits) = hits else {
        return Ok(());
    };
    writeln!(output, "  {label} ({})", hits.len())?;
    for hit in hits {
        writeln!(output, "    {}:{} [{}]", hit.file, hit.line, hit.attribute)?;
    }
    Ok(())
}

fn write_md(report: &DataPwReport, output: &mut dyn Write) -> io::Result<()> {
    writeln!(output, "# data-pw `{}`", report.value)?;
    write_section_md(output, "Source", report.source.as_deref())?;
    write_section_md(output, "Test", report.test.as_deref())
}

fn write_section_md(
    output: &mut dyn Write,
    label: &str,
    hits: Option<&[data_pw_query::DataPwHit]>,
) -> io::Result<()> {
    let Some(hits) = hits else {
        return Ok(());
    };
    writeln!(output, "\n## {label}")?;
    for hit in hits {
        writeln!(output, "- `{}:{}` ({})", hit.file, hit.line, hit.attribute)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
