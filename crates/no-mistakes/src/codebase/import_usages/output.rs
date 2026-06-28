use super::model::ImportUsagesReport;
use crate::cli::Format;
use anyhow::Result;
use std::io::{self, Write};

pub fn resolve_format(json: bool, format: Option<Format>, stdout_is_terminal: bool) -> Format {
    if json {
        Format::Json
    } else if let Some(format) = format {
        format
    } else if stdout_is_terminal {
        Format::Human
    } else {
        Format::Json
    }
}

pub fn write_report(
    report: &ImportUsagesReport,
    format: Format,
    out: &mut dyn Write,
) -> Result<()> {
    match format {
        Format::Json => {
            serde_json::to_writer_pretty(&mut *out, report)?;
            writeln!(out)?;
        }
        Format::Yml => {
            serde_yaml::to_writer(out, report)?;
        }
        Format::Md | Format::Human => write_human(report, out)?,
        Format::Paths => write_paths(report, out)?,
    }
    Ok(())
}

fn write_human(report: &ImportUsagesReport, out: &mut dyn Write) -> io::Result<()> {
    for file in &report.files {
        if file.imports.is_empty() {
            continue;
        }
        writeln!(out, "{}", file.path)?;
        for import in &file.imports {
            let package = import.package_name.as_deref().unwrap_or("-");
            writeln!(
                out,
                "  line {} {} {} package={}",
                import.line, import.kind, import.specifier, package
            )?;
        }
    }
    Ok(())
}

fn write_paths(report: &ImportUsagesReport, out: &mut dyn Write) -> io::Result<()> {
    for file in &report.files {
        if !file.imports.is_empty() {
            writeln!(out, "{}", file.path)?;
        }
    }
    Ok(())
}
