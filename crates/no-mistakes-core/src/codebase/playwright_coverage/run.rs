pub fn run(args: CoverageArgs) -> Result<ExitStatus> {
    let mut timings = crate::codebase::timing::PhaseTimings::start();
    let cwd = std::env::current_dir().context("reading current directory")?;
    let root = resolve_root(args.root.as_deref(), &cwd);
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let config = match load_config(&root) {
        Ok(config) => Some(config),
        Err(err) if args.frontend_root.is_some() => {
            eprintln!(
                "warning: ignoring guardrails config load error because --frontend-root was provided: {err:#}"
            );
            None
        }
        Err(err) => return Err(err).context("loading guardrails config"),
    };
    let frontend_root =
        resolve_frontend_root(args.frontend_root.as_deref(), &root, config.as_ref())?;

    timings.mark("search");

    let extra_skip = config
        .as_ref()
        .map(|config| config.filesystem.skip_directories.as_slice())
        .unwrap_or(&[]);
    let all_files = crate::codebase::ts_source::discover_files(&root, extra_skip);
    timings.mark("ingest");

    let report = collect_report_with_frontend_root(
        &root,
        &frontend_root,
        test_globs_or_default(&args.test_globs),
        &all_files,
    )?;
    timings.mark("parse+analysis");

    if report.summary.total == 0 {
        bail!(
            "no Next.js routes discovered under {}",
            frontend_root.display()
        );
    }

    let format = resolve_format(args.json, args.format, io::stdout().is_terminal());

    let stdout = io::stdout();
    let mut out = stdout.lock();
    write_report(&report, format, &mut out)?;

    timings.mark("output");
    if args.timings {
        timings.print_stderr();
    }

    if report.summary.uncovered == 0 {
        Ok(ExitStatus::Covered)
    } else {
        Ok(ExitStatus::Uncovered)
    }
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

pub(crate) fn collect_report_from_files(
    root: &Path,
    frontend_root: Option<&Path>,
    test_globs: &[String],
    all_files: &[PathBuf],
) -> Result<CoverageReport> {
    let config = load_config(root).context("loading guardrails config")?;
    let frontend_root = resolve_frontend_root(frontend_root, root, Some(&config))?;
    collect_report_with_frontend_root(
        root,
        &frontend_root,
        test_globs_or_default(test_globs),
        all_files,
    )
}
