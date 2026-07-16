use crate::cli::Format;
use anyhow::{Context, Result};
use serde::Serialize;
use std::io::{self, Write};

/// A renderable query report. JSON and YAML come for free from `Serialize`;
/// each command supplies the compact `human` tree and `paths` projection an
/// agent reads. `md` defaults to the human tree.
pub(crate) trait Report: Serialize {
    fn write_human(&self, w: &mut dyn Write) -> io::Result<()>;
    fn write_paths(&self, w: &mut dyn Write) -> io::Result<()>;
    /// Markdown defaults to the same compact tree as `human`.
    fn write_md(&self, w: &mut dyn Write) -> io::Result<()> {
        self.write_human(w)
    }
}

/// Write `report` in the requested `format` to `w`.
pub(crate) fn render<R: Report>(report: &R, format: Format, w: &mut dyn Write) -> Result<()> {
    render_with_deadline_check(report, format, w, crate::invocation::check_timeout)
}

fn render_with_deadline_check<R, F>(
    report: &R,
    format: Format,
    w: &mut dyn Write,
    mut check_deadline: F,
) -> Result<()>
where
    R: Report,
    F: FnMut() -> Result<()>,
{
    // Query analysis may stop scheduling work when the invocation expires. Do
    // not publish that potentially incomplete report before the CLI's final
    // deadline check observes the timeout.
    check_deadline()?;
    let mut output = Vec::new();
    match format {
        Format::Json => {
            serde_json::to_writer_pretty(&mut output, report).context("serializing query JSON")?;
            writeln!(output).context("writing query JSON")?;
        }
        Format::Yml => {
            serde_yaml::to_writer(&mut output, report).context("serializing query YAML")?
        }
        Format::Human => report
            .write_human(&mut output)
            .context("writing human output")?,
        Format::Paths => report
            .write_paths(&mut output)
            .context("writing paths output")?,
        Format::Md => report
            .write_md(&mut output)
            .context("writing markdown output")?,
    }
    check_deadline()?;
    w.write_all(&output).context("publishing query output")?;
    Ok(())
}

/// Serialize a report as pretty JSON (the N-API parity surface).
pub(crate) fn to_json<R: Report>(report: &R) -> Result<String> {
    serde_json::to_string_pretty(report).context("serializing query JSON")
}

/// Pick the effective format: `--json` wins, then `--format`, then human on a
/// TTY and JSON when piped.
pub(crate) fn resolve_format(json: bool, format: Option<Format>, is_tty: bool) -> Format {
    if json {
        Format::Json
    } else if let Some(format) = format {
        format
    } else if is_tty {
        Format::Human
    } else {
        Format::Json
    }
}

#[cfg(test)]
mod tests;
