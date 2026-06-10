use super::graph::{NodeEntry, NodeId};
use anyhow::Result;
use std::io::Write;
use std::path::Path;

include!("output_structured.rs");

/// Write findings as a JSON object: `{ "roots": [...], "files": [...] }`.
pub fn write_json(
    roots: &[String],
    entries: &[NodeEntry],
    root_dir: &Path,
    w: &mut dyn Write,
) -> Result<()> {
    let out = build_output(roots, entries, root_dir);
    serde_json::to_writer_pretty(&mut *w, &out)?;
    writeln!(w)?;
    Ok(())
}

/// Write one relative path per line — suitable for shell `$()` substitution.
/// QueueJob virtual nodes are rendered as `queueFile#job`.
pub fn write_paths(entries: &[NodeEntry], root_dir: &Path, w: &mut dyn Write) -> Result<()> {
    for entry in entries {
        match &entry.node {
            NodeId::File(p) => {
                let rel = p.strip_prefix(root_dir).unwrap_or(p);
                writeln!(w, "{}", rel.display())?;
            }
            NodeId::Symbol { file, symbol } => {
                let rel = file.strip_prefix(root_dir).unwrap_or(file);
                writeln!(w, "{}#{}", rel.display(), symbol)?;
            }
            NodeId::QueueJob { queue_file, job } => {
                let rel = queue_file
                    .strip_prefix(root_dir)
                    .unwrap_or(queue_file.as_path());
                writeln!(w, "{}#{}", rel.display(), job)?;
            }
            NodeId::Module(specifier) => {
                writeln!(w, "{specifier}")?;
            }
        }
    }
    Ok(())
}

/// Write a human-readable tree for TTY output.
pub fn write_human(
    roots: &[String],
    entries: &[NodeEntry],
    root_dir: &Path,
    w: &mut dyn Write,
) -> Result<()> {
    if roots.len() == 1 {
        writeln!(w, "{}", roots[0])?;
    } else {
        writeln!(w, "{} files", roots.len())?;
    }

    if entries.is_empty() {
        writeln!(w, "  (no results)")?;
        return Ok(());
    }

    for entry in entries {
        let name = entry.node.display_name(root_dir);
        let indent = "  ".repeat(entry.depth);
        writeln!(w, "{}{}", indent, name)?;
    }

    Ok(())
}

/// Write results as a Markdown nested bullet list.
pub fn write_md(
    roots: &[String],
    entries: &[NodeEntry],
    root_dir: &Path,
    w: &mut dyn Write,
) -> Result<()> {
    if roots.len() == 1 {
        writeln!(w, "# `{}`", roots[0])?;
    } else {
        writeln!(w, "# {} files", roots.len())?;
        for r in roots {
            writeln!(w, "- `{r}`")?;
        }
    }
    writeln!(w)?;

    if entries.is_empty() {
        writeln!(w, "_No results._")?;
        return Ok(());
    }

    for entry in entries {
        let name = entry.node.display_name(root_dir);
        let indent = "  ".repeat(entry.depth.saturating_sub(1));
        writeln!(w, "{}- `{}`", indent, name)?;
    }

    Ok(())
}

/// Write results as a YAML document with the same structure as JSON output.
pub fn write_yml(
    roots: &[String],
    entries: &[NodeEntry],
    root_dir: &Path,
    w: &mut dyn Write,
) -> Result<()> {
    let out = build_output(roots, entries, root_dir);
    let s = serde_yaml::to_string(&out)?;
    w.write_all(s.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests;
