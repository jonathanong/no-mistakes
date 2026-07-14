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
    extra_callers: &[CallerEntry],
    file_target_symbols: &BTreeMap<String, BTreeSet<String>>,
    facts: &TsFactMap,
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
        if has_file_level_import_edge(&entry.via) {
            let Some(target_symbols) = file_target_symbols
                .get(file.as_str())
                .filter(|symbols| !symbols.is_empty())
            else {
                continue;
            };
            if !file_entry_uses_any_symbol(root, file.as_str(), target_symbols, facts) {
                continue;
            }
        }
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
    for caller in extra_callers {
        by_file
            .entry(caller.file.clone())
            .and_modify(|existing| {
                existing.depth = existing.depth.min(caller.depth);
                merge_via(&mut existing.via, &caller.via);
            })
            .or_insert(TestSuggestion {
                file: caller.file.clone(),
                depth: caller.depth,
                via: caller.via.clone(),
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

fn export_location(
    facts: &TsFactMap,
    file: &Path,
    root: &Path,
    symbol: &str,
    allow_star_reexport: bool,
) -> Result<Option<SymbolLocation>> {
    let file_facts = facts
        .get(file)
        .with_context(|| format!("reading {}", file.display()))?;
    if let Some(parse_error) = &file_facts.parse_error {
        anyhow::bail!(
            "extracting symbols from {}: {parse_error}",
            file.display()
        );
    }
    let symbols = file_facts
        .symbols
        .as_ref()
        .with_context(|| format!("extracting symbols from {}", file.display()))?;
    Ok(symbols
        .exports
        .iter()
        .find(|export| export_matches_symbol(&export.kind, &export.name, symbol, allow_star_reexport))
        .map(|export| SymbolLocation {
            file: relative_slash_path(root, file),
            symbol: symbol.to_string(),
            line: export.line,
            kind: export_kind_str(&export.kind),
        }))
}

fn export_matches_symbol(
    kind: &ExportKind,
    name: &str,
    symbol: &str,
    allow_star_reexport: bool,
) -> bool {
    if matches!(
        kind,
        ExportKind::ReExport { imported, .. } if imported == "*" && name == "*"
    ) {
        return allow_star_reexport;
    }
    export_name(kind, name) == symbol
}

fn export_name<'a>(kind: &ExportKind, name: &'a str) -> &'a str {
    if matches!(kind, ExportKind::Default) {
        "default"
    } else {
        name
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

fn via_strings(via: &[EdgeKind]) -> Vec<&'static str> {
    via.iter().map(EdgeKind::as_str).collect()
}

fn merge_via(target: &mut Vec<&'static str>, source: &[&'static str]) {
    target.extend(source.iter().copied());
    target.sort_unstable();
    target.dedup();
}

#[cfg(test)]
mod impact_output_tests;

include!("impact_text.rs");
