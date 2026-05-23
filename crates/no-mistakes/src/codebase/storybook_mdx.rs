use crate::codebase::storybook::{
    StorybookFileFacts, StorybookSideEffectImport, UsedRuntimeImport,
};

pub(crate) fn extract_mdx_source(source: &str) -> StorybookFileFacts {
    let mut used_runtime_imports = Vec::new();
    let mut side_effect_imports = Vec::new();
    let mut pending_import: Option<(String, u32)> = None;
    let mut active_code_fence: Option<&'static str> = None;
    let mut reference_source = String::new();
    for (index, line) in source.lines().enumerate() {
        let line_number = (index + 1) as u32;
        let trimmed = line.trim();
        if let Some(fence) = active_code_fence {
            if trimmed.starts_with(fence) {
                active_code_fence = None;
            }
            continue;
        }
        if trimmed.starts_with("```") {
            active_code_fence = Some("```");
            continue;
        }
        if trimmed.starts_with("~~~") {
            active_code_fence = Some("~~~");
            continue;
        }
        if let Some((pending, _)) = pending_import.as_mut() {
            if trimmed.starts_with("import ") {
                pending_import = None;
            } else {
                pending.push(' ');
                pending.push_str(trimmed);
                if mdx_import_complete(pending) {
                    let (import, line) = pending_import.take().expect("pending import exists");
                    push_mdx_import_line(
                        &mut used_runtime_imports,
                        &mut side_effect_imports,
                        &import,
                        line,
                    );
                }
                continue;
            }
        }
        let Some(import) = trimmed.strip_prefix("import ") else {
            reference_source.push_str(line);
            reference_source.push('\n');
            continue;
        };
        let import = import.trim();
        if import.starts_with("type ") {
            continue;
        }
        if mdx_import_complete(import) {
            push_mdx_import_line(
                &mut used_runtime_imports,
                &mut side_effect_imports,
                import,
                line_number,
            );
        } else {
            pending_import = Some((import.to_string(), line_number));
        }
    }
    used_runtime_imports.retain(|import| mdx_references_local(&reference_source, &import.local));
    StorybookFileFacts {
        used_runtime_imports,
        side_effect_imports,
    }
}

fn mdx_references_local(source: &str, local: &str) -> bool {
    source.contains(&format!("<{local}"))
        || source.contains(&format!("{{{local}"))
        || source.contains(&format!("{{ {local}"))
}

fn mdx_import_complete(import: &str) -> bool {
    side_effect_source(import.trim()).is_some() || import.contains(" from ")
}

fn push_mdx_import_line(
    used_runtime_imports: &mut Vec<UsedRuntimeImport>,
    side_effect_imports: &mut Vec<StorybookSideEffectImport>,
    import: &str,
    line: u32,
) {
    if let Some(source) = side_effect_source(import.trim()) {
        side_effect_imports.push(StorybookSideEffectImport {
            source: source.to_string(),
            line,
        });
        return;
    }
    let Some((clause, from)) = import.split_once(" from ") else {
        return;
    };
    let Some(source) = quoted_import_source(from.trim().trim_end_matches(';')) else {
        return;
    };
    push_mdx_imports(used_runtime_imports, clause.trim(), source, line);
}

fn push_mdx_imports(imports: &mut Vec<UsedRuntimeImport>, clause: &str, source: &str, line: u32) {
    if clause.starts_with("type ") {
        return;
    }
    if let Some((default, rest)) = clause.split_once(", {") {
        push_mdx_default_import(imports, default.trim(), source, line);
        let named = format!("{{{}", rest.trim());
        push_mdx_imports(imports, &named, source, line);
        return;
    }
    if let Some(namespace) = clause.strip_prefix("* as ") {
        let local = namespace.trim();
        if !local.is_empty() {
            imports.push(UsedRuntimeImport {
                source: source.to_string(),
                imported: "*".to_string(),
                local: local.to_string(),
                namespace: true,
                line,
            });
        }
        return;
    }
    if clause.starts_with('{') && clause.ends_with('}') {
        for specifier in clause[1..clause.len() - 1].split(',') {
            push_mdx_named_import(imports, specifier.trim(), source, line);
        }
        return;
    }
    if clause.starts_with('{') || clause.ends_with('}') {
        return;
    }
    push_mdx_default_import(imports, clause, source, line);
}

fn push_mdx_default_import(
    imports: &mut Vec<UsedRuntimeImport>,
    local: &str,
    source: &str,
    line: u32,
) {
    if local.is_empty() {
        return;
    }
    imports.push(UsedRuntimeImport {
        source: source.to_string(),
        imported: "default".to_string(),
        local: local.to_string(),
        namespace: false,
        line,
    });
}

fn push_mdx_named_import(
    imports: &mut Vec<UsedRuntimeImport>,
    specifier: &str,
    source: &str,
    line: u32,
) {
    if specifier.is_empty() || specifier.starts_with("type ") {
        return;
    }
    let (imported, local) = specifier
        .split_once(" as ")
        .map_or((specifier, specifier), |(imported, local)| {
            (imported.trim(), local.trim())
        });
    if imported.is_empty() || local.is_empty() {
        return;
    }
    imports.push(UsedRuntimeImport {
        source: source.to_string(),
        imported: imported.to_string(),
        local: local.to_string(),
        namespace: false,
        line,
    });
}

fn side_effect_source(value: &str) -> Option<&str> {
    quoted_import_source(value.trim_end_matches(';'))
}

fn quoted_import_source(value: &str) -> Option<&str> {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
}

#[cfg(test)]
mod tests;
