#[derive(Serialize)]
struct ImportClosureOutput<'a> {
    files: Vec<String>,
    diagnostics: &'a [crate::codebase::ts_resolver::TsConfigDiagnostic],
}

pub(crate) fn write_import_closure_json(
    entries: &[NodeEntry],
    root_dir: &Path,
    diagnostics: &[crate::codebase::ts_resolver::TsConfigDiagnostic],
    w: &mut dyn Write,
) -> Result<()> {
    let mut files = entries
        .iter()
        .filter_map(|entry| match &entry.node {
            NodeId::File(path) => {
                let file = path.as_ref();
                let rel = file.strip_prefix(root_dir).unwrap_or(file);
                Some(rel.to_string_lossy().replace('\\', "/"))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    files.sort();
    files.dedup();
    serde_json::to_writer(&mut *w, &ImportClosureOutput { files, diagnostics })?;
    writeln!(w)?;
    Ok(())
}
