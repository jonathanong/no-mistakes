use super::Output;
use anyhow::Result;
use std::io::Write;

pub(super) fn write_paths(report: &Output, w: &mut dyn Write) -> Result<()> {
    for file in &report.files {
        for row in &file.exports {
            writeln!(w, "{}:{}:{}", file.path, row.line, row.name)?;
        }
        for row in &file.imports {
            writeln!(w, "{}:{}:{}", file.path, row.line, row.local)?;
        }
    }
    Ok(())
}

pub(super) fn write_human(report: &Output, w: &mut dyn Write) -> Result<()> {
    if report.roots.len() == 1 {
        writeln!(w, "{}", report.roots[0])?;
    } else {
        writeln!(w, "{} files", report.roots.len())?;
    }
    if report
        .files
        .iter()
        .all(|file| file.exports.is_empty() && file.imports.is_empty())
    {
        writeln!(w, "  (no symbols)")?;
        return Ok(());
    }
    for (index, file) in report.files.iter().enumerate() {
        if report.files.len() > 1 {
            if index > 0 {
                writeln!(w)?;
            }
            writeln!(w, "{}", file.path)?;
        }
        for row in &file.exports {
            writeln!(
                w,
                "  export {:<12} {:<24} :{}",
                row.kind, row.name, row.line
            )?;
        }
        for row in &file.imports {
            writeln!(
                w,
                "  import {:<24} :{:<4} from {}",
                row.local, row.line, row.source
            )?;
        }
    }
    Ok(())
}

pub(super) fn write_md(report: &Output, w: &mut dyn Write) -> Result<()> {
    if report.roots.len() == 1 {
        writeln!(w, "# `{}`", report.roots[0])?;
    } else {
        writeln!(w, "# {} files", report.roots.len())?;
    }
    writeln!(w)?;
    if report
        .files
        .iter()
        .all(|file| file.exports.is_empty() && file.imports.is_empty())
    {
        writeln!(w, "_No symbols found._")?;
        return Ok(());
    }
    for file in &report.files {
        if report.files.len() > 1 {
            writeln!(w, "## `{}`", file.path)?;
        }
        if !file.exports.is_empty() {
            writeln!(w, "### Exports")?;
            for row in &file.exports {
                writeln!(w, "- `{}` ({}, line {})", row.name, row.kind, row.line)?;
            }
        }
        if !file.imports.is_empty() {
            writeln!(w, "### Imports")?;
            for row in &file.imports {
                writeln!(
                    w,
                    "- `{}` from `{}` (line {})",
                    row.local, row.source, row.line
                )?;
            }
        }
    }
    Ok(())
}
