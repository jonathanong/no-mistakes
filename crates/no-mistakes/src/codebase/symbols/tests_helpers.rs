fn resolve_tsconfig(arg: Option<&Path>, root: &Path) -> Result<TsConfig> {
    crate::codebase::ts_resolver::resolve_tsconfig(arg, root)
}

fn fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/symbols-output/fixture"),
    )
}

/// Render `args` through the same `collect_entries` pipeline production uses,
/// then serialize via the requested formatter into a String.
fn run_capture(args: SymbolsArgs) -> String {
    if args.mode == SymbolsMode::SignatureImpact {
        let report = impact::collect_report(&args).unwrap();
        let mut buf = Vec::new();
        let format = args.format.unwrap_or(Format::Json);
        impact::write_report(&report, format, &mut buf).unwrap();
        return String::from_utf8(buf).unwrap();
    }
    let (entries, root_strs) = collect_entries(&args).unwrap();
    let mut buf = Vec::new();
    let format = args.format.unwrap_or(Format::Json);
    match format {
        Format::Json => output::write_json(&root_strs, &entries, &mut buf).unwrap(),
        Format::Md => output::write_md(&root_strs, &entries, &mut buf).unwrap(),
        Format::Yml => output::write_yml(&root_strs, &entries, &mut buf).unwrap(),
        Format::Paths => output::write_paths(&entries, &mut buf).unwrap(),
        Format::Human => output::write_human(&root_strs, &entries, &mut buf).unwrap(),
    }
    String::from_utf8(buf).unwrap()
}

fn args_for(root: &Path, files: Vec<&str>, format: Format) -> SymbolsArgs {
    SymbolsArgs {
        files: files.into_iter().map(PathBuf::from).collect(),
        root: Some(root.to_path_buf()),
        tsconfig: None,
        config: None,
        mode: SymbolsMode::List,
        symbol: None,
        kinds: vec![],
        include: Include::Exports,
        format: Some(format),
        json: false,
        timings: false,
    }
}

fn fixture_args(files: Vec<&str>, format: Format) -> SymbolsArgs {
    args_for(&fixture_root(), files, format)
}
