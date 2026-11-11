fn ensure_signature_impact_target_analyzed(
    context: &PreparedReportContext<'_>,
    target_file: &Path,
) -> Result<()> {
    if let Some(reason) = context.facts.get(target_file).and_then(|facts| {
        facts
            .operational_error
            .as_ref()
            .or(facts.parse_error.as_ref())
    }) {
        ensure_signature_impact_failure_unrelated(
            &UnanalyzedFile {
                file: relative_slash_path(context.root, target_file),
                reason: reason.clone(),
            },
            true,
        )?;
    }
    Ok(())
}

fn signature_impact_unanalyzed(
    context: &PreparedReportContext<'_>,
    target_file: &Path,
    exports: &[SymbolLocation],
) -> Result<Vec<UnanalyzedFile>> {
    let mut failures = context
        .facts
        .iter()
        .filter_map(|(path, facts)| {
            facts
                .operational_error
                .as_ref()
                .or(facts.parse_error.as_ref())
                .map(|reason| {
                    (
                        path,
                        facts,
                        UnanalyzedFile {
                            file: relative_slash_path(context.root, path),
                            reason: reason.clone(),
                        },
                    )
                })
        })
        .collect::<Vec<_>>();
    failures.sort_by(|left, right| left.2.file.cmp(&right.2.file));
    if failures.is_empty() {
        return Ok(Vec::new());
    }
    let names = exports
        .iter()
        .map(|export| export.symbol.clone())
        .collect::<BTreeSet<_>>();
    let relevant = signature_impact_relevant_files(context, target_file, &names);
    for (path, facts, failure) in &failures {
        ensure_signature_impact_failure_unrelated(
            failure,
            path.as_path() == target_file
                || relevant.contains(path.as_path())
                || facts.source.is_none()
                || facts
                    .source
                    .as_ref()
                    .is_some_and(|source| names.iter().any(|name| source.contains(name.as_str()))),
        )?;
    }
    Ok(failures
        .into_iter()
        .map(|(_, _, failure)| failure)
        .collect())
}

fn ensure_signature_impact_failure_unrelated(
    failure: &UnanalyzedFile,
    potentially_related: bool,
) -> Result<()> {
    if potentially_related {
        bail!(
            "signature-impact could not analyze `{}`: {}",
            failure.file,
            failure.reason
        );
    }
    Ok(())
}

fn signature_impact_relevant_files(
    context: &PreparedReportContext<'_>,
    target_file: &Path,
    names: &BTreeSet<String>,
) -> BTreeSet<PathBuf> {
    let target = NodeId::file_in(context.session.interner(), target_file);
    let allowed = signature_impact_edges();
    let mut roots = vec![target.clone()];
    roots.extend(context.facts.iter().filter_map(|(path, facts)| {
        let imports_target = facts.symbols.as_ref().is_some_and(|symbols| {
            symbols.imports.iter().any(|import| {
                names.contains(&import.imported)
                    || (import.imported == "*"
                        && facts.source.as_ref().is_some_and(|source| {
                            names.iter().any(|name| source.contains(name.as_str()))
                        }))
            }) || symbols.exports.iter().any(|export| {
                matches!(&export.kind, ExportKind::ReExport { imported, .. }
                    if names.contains(imported) || names.contains(&export.name))
            })
        });
        imports_target.then(|| NodeId::file_in(context.session.interner(), path))
    }));
    roots.extend(
        context
            .graph
            .dependents_of(&[target], None, Some(&allowed))
            .into_iter()
            .map(|entry| entry.node),
    );
    let mut relevant = roots
        .iter()
        .filter_map(NodeId::as_file)
        .map(Path::to_path_buf)
        .collect::<BTreeSet<_>>();
    relevant.extend(
        context
            .graph
            .deps_of(&roots, None, Some(&allowed))
            .into_iter()
            .filter_map(|entry| entry.node.as_file().map(Path::to_path_buf)),
    );
    relevant
}

fn signature_impact_warnings(
    suggested_tests: &[TestSuggestion],
    unanalyzed: &[UnanalyzedFile],
) -> Vec<ImpactWarning> {
    let mut warnings = warnings(suggested_tests);
    if !unanalyzed.is_empty() {
        warnings.push(ImpactWarning {
            r#type: "unanalyzed-files",
            message: format!(
                "{} file(s) could not be analyzed; inspect unanalyzed for details",
                unanalyzed.len()
            ),
        });
    }
    warnings
}
