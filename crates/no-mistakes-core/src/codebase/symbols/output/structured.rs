pub fn write_json(roots: &[String], entries: &[FileEntry], w: &mut dyn Write) -> Result<()> {
    let out = build_output(roots, entries);
    serde_json::to_writer_pretty(&mut *w, &out)?;
    writeln!(w)?;
    Ok(())
}

pub fn write_yml(roots: &[String], entries: &[FileEntry], w: &mut dyn Write) -> Result<()> {
    let out = build_output(roots, entries);
    serde_yaml::to_writer(w, &out)?;
    Ok(())
}

pub fn write_md(roots: &[String], entries: &[FileEntry], w: &mut dyn Write) -> Result<()> {
    if roots.len() == 1 {
        writeln!(w, "# `{}`", roots[0])?;
    } else {
        writeln!(w, "# {} files", roots.len())?;
        for root in roots {
            writeln!(w, "- `{root}`")?;
        }
    }
    writeln!(w)?;

    if entries
        .iter()
        .all(|e| e.exports.is_empty() && e.imports.is_empty())
    {
        writeln!(w, "_No symbols found._")?;
        return Ok(());
    }

    for entry in entries {
        if entries.len() > 1 {
            writeln!(w, "## `{}`", entry.rel_path.display())?;
        }
        for heading in ["### Exports"]
            .iter()
            .take(usize::from(!entry.exports.is_empty()))
        {
            writeln!(w, "{heading}")?;
        }
        for e in &entry.exports {
            let kind = export_kind_str(&e.kind);
            if let ExportKind::ReExport { source, imported } = &e.kind {
                let src = display_source(&e.resolved, source);
                let line = format!(
                    "- `{}` ({}, line {}) - re-exports `{}` from `{}`",
                    e.name, kind, e.line, imported, src
                );
                writeln!(w, "{line}")?;
            } else {
                writeln!(w, "- `{}` ({}, line {})", e.name, kind, e.line)?;
            }
        }
        if !entry.imports.is_empty() {
            writeln!(w, "### Imports")?;
            for i in &entry.imports {
                let type_tag = if i.is_type_only { " (type-only)" } else { "" };
                let src = display_source(&i.resolved, &i.source);
                if i.imported == i.local {
                    let line = format!(
                        "- `{}` from `{}` (line {}){}",
                        i.imported, src, i.line, type_tag
                    );
                    writeln!(w, "{line}")?;
                } else {
                    let line = format!(
                        "- `{}` as `{}` from `{}` (line {}){}",
                        i.imported, i.local, src, i.line, type_tag
                    );
                    writeln!(w, "{line}")?;
                }
            }
            writeln!(w)?;
        }
    }
    Ok(())
}
