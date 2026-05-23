fn write_report(report: &CoverageReport, format: Format, out: &mut dyn Write) -> Result<()> {
    match format {
        Format::Json => write_json(report, out),
        Format::Md => write_markdown(report, out),
        Format::Yml => write_yml(report, out),
        Format::Paths => write_paths(report, out),
        Format::Human => write_human(report, out),
    }
}

fn write_json(report: &CoverageReport, out: &mut dyn Write) -> Result<()> {
    serde_json::to_writer_pretty(&mut *out, report)
        .context("serializing coverage report to JSON")?;
    writeln!(out)?;
    Ok(())
}

fn write_yml(report: &CoverageReport, out: &mut dyn Write) -> Result<()> {
    let yml = serde_yaml::to_string(report).context("serializing coverage report to YAML")?;
    out.write_all(yml.as_bytes())?;
    Ok(())
}

fn write_paths(report: &CoverageReport, out: &mut dyn Write) -> Result<()> {
    for route in report.routes.iter().filter(|route| !route.covered) {
        writeln!(out, "{}", route.file)?;
    }
    Ok(())
}

fn write_human(report: &CoverageReport, out: &mut dyn Write) -> Result<()> {
    let line = format!(
        "Playwright route coverage: {}/{} ({:.1}%)",
        report.summary.covered, report.summary.total, report.summary.coverage_percent
    );
    writeln!(out, "{line}")?;

    if report.summary.uncovered == 0 {
        writeln!(out, "All routes are covered.")?;
        return Ok(());
    }

    writeln!(out, "Uncovered routes:")?;
    for route in report.routes.iter().filter(|route| !route.covered) {
        writeln!(out, "  {} ({})", route.route, route.file)?;
    }
    Ok(())
}

fn write_markdown(report: &CoverageReport, out: &mut dyn Write) -> Result<()> {
    let header = format!(
        "# Playwright route coverage\n\n- Covered: {}/{}\n- Coverage: {:.1}%\n",
        report.summary.covered, report.summary.total, report.summary.coverage_percent
    );
    writeln!(out, "{header}")?;

    if report.summary.uncovered == 0 {
        writeln!(out, "_All routes are covered._")?;
        return Ok(());
    }

    writeln!(out, "## Uncovered routes\n")?;
    for route in report.routes.iter().filter(|route| !route.covered) {
        writeln!(out, "- `{}` ({})", route.route, route.file)?;
    }
    Ok(())
}

