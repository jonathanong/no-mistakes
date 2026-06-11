pub fn write_report(
    report: &SignatureImpactReport,
    format: Format,
    out: &mut dyn Write,
) -> Result<()> {
    match format {
        Format::Json => {
            serde_json::to_writer_pretty(&mut *out, report)?;
            writeln!(out)?;
        }
        Format::Yml => serde_yaml::to_writer(out, report)?,
        Format::Paths => {
            for test in &report.suggested_tests {
                writeln!(out, "{}", test.file)?;
            }
        }
        Format::Md => write_markdown(report, out)?,
        Format::Human => write_human(report, out)?,
    }
    Ok(())
}

pub fn report_json(args: SymbolsArgs) -> Result<String> {
    let report = collect_report(&args)?;
    let mut out = Vec::new();
    write_report(&report, Format::Json, &mut out)?;
    String::from_utf8(out).context("symbols signature-impact JSON output must be UTF-8")
}

fn suggested_tests(
    entries: &[NodeEntry],
    root: &Path,
    test_filter: &TestFileFilter,
) -> Vec<TestSuggestion> {
    let mut by_file: BTreeMap<String, TestSuggestion> = BTreeMap::new();
    for entry in entries {
        let Some(path) = entry.node.as_file() else {
            continue;
        };
        if !test_filter.is_match(root, path) {
            continue;
        }
        let file = relative_slash_path(root, path);
        let via = via_strings(&entry.via);
        by_file
            .entry(file.clone())
            .and_modify(|existing| {
                existing.depth = existing.depth.min(entry.depth);
                merge_via(&mut existing.via, &via);
            })
            .or_insert(TestSuggestion {
                file,
                depth: entry.depth,
                via,
            });
    }
    let mut tests: Vec<_> = by_file.into_values().collect();
    tests.sort_by(|a, b| (a.depth, a.file.as_str()).cmp(&(b.depth, b.file.as_str())));
    tests
}

fn warnings(suggested_tests: &[TestSuggestion]) -> Vec<ImpactWarning> {
    if suggested_tests.is_empty() {
        vec![ImpactWarning {
            r#type: "no-suggested-tests",
            message: "No test files were reachable from this symbol.".to_string(),
        }]
    } else {
        Vec::new()
    }
}

fn export_location(file: &Path, root: &Path, symbol: &str) -> Result<Option<SymbolLocation>> {
    let source =
        std::fs::read_to_string(file).with_context(|| format!("reading {}", file.display()))?;
    let is_tsx = matches!(
        file.extension().and_then(|s| s.to_str()),
        Some("tsx") | Some("jsx")
    );
    let symbols = extract_symbols(&source, is_tsx)
        .with_context(|| format!("extracting symbols from {}", file.display()))?;
    Ok(symbols
        .exports
        .into_iter()
        .find(|export| export_name(&export.kind, &export.name) == symbol)
        .map(|export| SymbolLocation {
            file: relative_slash_path(root, file),
            symbol: symbol.to_string(),
            line: export.line,
            kind: export_kind_str(&export.kind),
        }))
}

fn export_name(kind: &ExportKind, name: &str) -> String {
    if matches!(kind, ExportKind::Default) {
        "default".to_string()
    } else {
        name.to_string()
    }
}

fn caller_parts(node: &NodeId, root: &Path) -> Option<(String, Option<String>)> {
    match node {
        NodeId::File(path) => Some((relative_slash_path(root, path), None)),
        NodeId::Symbol { file, symbol } => {
            Some((relative_slash_path(root, file), Some(symbol.clone())))
        }
        NodeId::Module(_) | NodeId::QueueJob { .. } => None,
    }
}

fn node_name(node: &NodeId, root: &Path) -> String {
    match node {
        NodeId::File(path) => relative_slash_path(root, path),
        NodeId::Symbol { file, symbol } => {
            format!("{}#{}", relative_slash_path(root, file), symbol)
        }
        NodeId::Module(specifier) => specifier.clone(),
        NodeId::QueueJob { queue_file, job } => {
            format!("{}#{}", relative_slash_path(root, queue_file), job)
        }
    }
}

fn via_strings(via: &[EdgeKind]) -> Vec<&'static str> {
    via.iter().map(EdgeKind::as_str).collect()
}

fn merge_via(target: &mut Vec<&'static str>, source: &[&'static str]) {
    target.extend(source.iter().copied());
    target.sort_unstable();
    target.dedup();
}

include!("impact_text.rs");
