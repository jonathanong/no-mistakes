pub fn write_paths(entries: &[FileEntry], w: &mut dyn Write) -> Result<()> {
    for entry in entries {
        let path = entry.rel_path.display();
        for e in &entry.exports {
            writeln!(w, "{}:{}:{}", path, e.line, e.name)?;
        }
        for i in &entry.imports {
            writeln!(w, "{}:{}:{}", path, i.line, i.local)?;
        }
    }
    Ok(())
}

pub fn write_human(roots: &[String], entries: &[FileEntry], w: &mut dyn Write) -> Result<()> {
    if roots.len() == 1 {
        writeln!(w, "{}", roots[0])?;
    } else {
        writeln!(w, "{} files", roots.len())?;
    }

    if entries
        .iter()
        .all(|e| e.exports.is_empty() && e.imports.is_empty())
    {
        writeln!(w, "  (no symbols)")?;
        return Ok(());
    }

    for (idx, entry) in entries.iter().enumerate() {
        if entries.len() > 1 {
            if idx > 0 {
                writeln!(w)?;
            }
            writeln!(w, "{}", entry.rel_path.display())?;
        }
        for e in &entry.exports {
            let kind = export_kind_str(&e.kind);
            match &e.kind {
                ExportKind::ReExport { source, imported } => {
                    let src = display_source(&e.resolved, source);
                    let line = format!(
                        "  export {:<10} {:<24} :{:<4} <- {} from {}",
                        kind, e.name, e.line, imported, src
                    );
                    writeln!(w, "{line}")?;
                }
                _ => {
                    writeln!(w, "  export {:<10} {:<24} :{}", kind, e.name, e.line)?;
                }
            }
        }
        for i in &entry.imports {
            let type_tag = if i.is_type_only { " (type)" } else { "" };
            let lhs = if i.imported == i.local {
                i.imported.clone()
            } else {
                format!("{} as {}", i.imported, i.local)
            };
            let src = display_source(&i.resolved, &i.source);
            let line = format!(
                "  import{} {:<24} :{:<4} from {}",
                type_tag, lhs, i.line, src
            );
            writeln!(w, "{line}")?;
        }
    }
    Ok(())
}
