pub fn run(args: SymbolsArgs) -> Result<()> {
    let _diagnostics = crate::diagnostics::LegacyDiagnosticsGuard::new(args.timings, false);
    let mut timings = crate::codebase::timing::PhaseTimings::start();
    if args.mode == SymbolsMode::SignatureImpact {
        let report = impact::collect_report(&args)?;
        timings.mark("parse+analysis");
        let format = resolve_format(args.json, args.format, io::stdout().is_terminal());
        let stdout = io::stdout();
        let mut out = stdout.lock();
        impact::write_report(&report, format, &mut out)?;
        timings.mark("output");
        if args.timings {
            timings.print_stderr();
        }
        return Ok(());
    }
    let (entries, root_strs) = collect_entries_with_timings(&args, Some(&mut timings))?;
    let format = resolve_format(args.json, args.format, io::stdout().is_terminal());
    let stdout = io::stdout();
    let mut out = stdout.lock();
    match format {
        Format::Json => output::write_json(&root_strs, &entries, &mut out)?,
        Format::Md => output::write_md(&root_strs, &entries, &mut out)?,
        Format::Yml => output::write_yml(&root_strs, &entries, &mut out)?,
        Format::Paths => output::write_paths(&entries, &mut out)?,
        Format::Human => output::write_human(&root_strs, &entries, &mut out)?,
    }
    timings.mark("output");
    if args.timings {
        timings.print_stderr();
    }
    Ok(())
}

pub fn run_json(args: SymbolsArgs) -> Result<String> {
    if args.mode == SymbolsMode::SignatureImpact {
        return impact::report_json(args);
    }
    let (entries, root_strs) = collect_entries(&args)?;
    let mut out = Vec::new();
    output::write_json(&root_strs, &entries, &mut out)?;
    String::from_utf8(out).context("symbols JSON output must be UTF-8")
}

fn resolve_format(json: bool, format: Option<Format>, stdout_is_terminal: bool) -> Format {
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
