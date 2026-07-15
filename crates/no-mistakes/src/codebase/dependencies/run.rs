pub fn run(args: TraverseArgs, direction: Direction) -> Result<()> {
    let _diagnostics = crate::diagnostics::LegacyDiagnosticsGuard::new(args.timings, false);
    let cwd_early = std::env::current_dir().context("reading current directory")?;
    let mut timings = crate::codebase::timing::PhaseTimings::start();

    let result = collect_and_filter_entries(&args, direction, &cwd_early, &mut timings)?;
    let output_result = output_results(&args, &result);

    timings.mark("output");
    if args.timings {
        timings.print_stderr();
    }

    output_result?;
    Ok(())
}

pub fn run_json(args: TraverseArgs, direction: Direction) -> Result<String> {
    let cwd_early = std::env::current_dir().context("reading current directory")?;
    let mut timings = crate::codebase::timing::PhaseTimings::start();
    let result = collect_and_filter_entries(&args, direction, &cwd_early, &mut timings)?;
    let root_strings = output_root_strings(&args);
    let mut out = Vec::new();
    write_output_results(Format::Json, &root_strings, &result, &mut out)?;
    String::from_utf8(out).context("dependency JSON output must be UTF-8")
}

pub(crate) fn result_json(args: &TraverseArgs, result: &TraversalResult) -> Result<String> {
    let root_strings = output_root_strings(args);
    let mut out = Vec::new();
    write_output_results(Format::Json, &root_strings, result, &mut out)?;
    String::from_utf8(out).context("dependency JSON output must be UTF-8")
}

fn output_results(args: &TraverseArgs, result: &TraversalResult) -> Result<()> {
    let root_strings = output_root_strings(args);
    let stdout = io::stdout();
    let stdout_is_terminal = stdout.is_terminal();
    let mut out = stdout.lock();
    let format = resolve_format(args.json, args.format, stdout_is_terminal);
    write_output_results(format, &root_strings, result, &mut out)
}

fn output_root_strings(args: &TraverseArgs) -> Vec<String> {
    args.files
        .iter()
        .enumerate()
        .map(|(index, file)| {
            let file = file.display().to_string();
            match args.file_symbols.get(index).and_then(Option::as_deref) {
                Some(symbol) => format!("{file}#{symbol}"),
                None => file,
            }
        })
        .collect()
}
