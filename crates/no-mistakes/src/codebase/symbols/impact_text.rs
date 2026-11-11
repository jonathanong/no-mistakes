fn write_markdown(report: &SignatureImpactReport, out: &mut dyn Write) -> Result<()> {
    let title = report
        .roots
        .first()
        .map(String::as_str)
        .unwrap_or(report.symbol.as_str());
    writeln!(out, "# `{title}`")?;
    writeln!(out)?;
    writeln!(out, "- Defined in: `{}` (line {})", report.definition.file, report.definition.line)?;
    write_location_section("Exported via", &report.exports, out)?;
    write_caller_section("Production callers", &report.production_callers, out)?;
    write_caller_section("Test callers", &report.test_callers, out)?;
    writeln!(out, "## Suggested tests")?;
    if report.suggested_tests.is_empty() {
        writeln!(out, "_No suggested tests found._")?;
    } else {
        for test in &report.suggested_tests {
            writeln!(out, "- `{}`", test.file)?;
        }
    }
    write_unanalyzed_section(report, true, out)?;
    Ok(())
}

fn write_human(report: &SignatureImpactReport, out: &mut dyn Write) -> Result<()> {
    writeln!(out, "Symbol: {}", report.symbol)?;
    writeln!(out, "Defined in: {}:{}", report.definition.file, report.definition.line)?;
    write_location_section("Exported via", &report.exports, out)?;
    write_caller_section("Production callers", &report.production_callers, out)?;
    write_caller_section("Test callers", &report.test_callers, out)?;
    writeln!(out, "Suggested tests:")?;
    if report.suggested_tests.is_empty() {
        writeln!(out, "  (none)")?;
    } else {
        for test in &report.suggested_tests {
            writeln!(out, "  {}", test.file)?;
        }
    }
    write_unanalyzed_section(report, false, out)?;
    Ok(())
}

fn write_location_section(
    heading: &str,
    locations: &[SymbolLocation],
    out: &mut dyn Write,
) -> Result<()> {
    writeln!(out, "## {heading}")?;
    for location in locations {
            writeln!(out, "- `{}#{}` ({}, line {})", location.file, location.symbol, location.kind, location.line)?;
    }
    Ok(())
}

fn write_caller_section(heading: &str, callers: &[CallerEntry], out: &mut dyn Write) -> Result<()> {
    writeln!(out, "## {heading}")?;
    if callers.is_empty() {
        writeln!(out, "_None._")?;
    } else {
        for caller in callers {
            if let Some(symbol) = &caller.symbol {
                writeln!(out, "- `{}#{}`", caller.file, symbol)?;
            } else {
                writeln!(out, "- `{}`", caller.file)?;
            }
        }
    }
    Ok(())
}

fn write_unanalyzed_section(
    report: &SignatureImpactReport,
    markdown: bool,
    out: &mut dyn Write,
) -> Result<()> {
    if report.unanalyzed.is_empty() {
        return Ok(());
    }
    writeln!(out, "{}", if markdown { "## Unanalyzed files" } else { "Unanalyzed files:" })?;
    for warning in report.warnings.iter().filter(|warning| warning.r#type == "unanalyzed-files") {
        writeln!(out, "Warning [{}]: {}", warning.r#type, warning.message)?;
    }
    for failure in &report.unanalyzed {
        if markdown {
            writeln!(out, "- `{}`: {}", failure.file, failure.reason)?;
        } else {
            writeln!(out, "  {}: {}", failure.file, failure.reason)?;
        }
    }
    Ok(())
}
