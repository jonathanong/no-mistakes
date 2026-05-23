use crate::codebase::storybook::{
    StorybookFileFacts, StorybookSideEffectImport, UsedRuntimeImport,
};

pub(crate) fn extract_mdx_source(source: &str) -> StorybookFileFacts {
    let mut used_runtime_imports = Vec::new();
    let mut side_effect_imports = Vec::new();
    for (index, line) in source.lines().enumerate() {
        let line_number = (index + 1) as u32;
        let trimmed = line.trim();
        let Some(import) = trimmed.strip_prefix("import ") else {
            continue;
        };
        if let Some(source) = side_effect_source(import.trim()) {
            side_effect_imports.push(StorybookSideEffectImport {
                source: source.to_string(),
                line: line_number,
            });
            continue;
        }
        let Some((clause, from)) = import.split_once(" from ") else {
            continue;
        };
        let Some(source) = quoted_import_source(from.trim().trim_end_matches(';')) else {
            continue;
        };
        push_mdx_imports(
            &mut used_runtime_imports,
            clause.trim(),
            source,
            line_number,
        );
    }
    StorybookFileFacts {
        used_runtime_imports,
        side_effect_imports,
    }
}

fn push_mdx_imports(imports: &mut Vec<UsedRuntimeImport>, clause: &str, source: &str, line: u32) {
    if let Some((default, rest)) = clause.split_once(',') {
        push_mdx_default_import(imports, default.trim(), source, line);
        push_mdx_imports(imports, rest.trim(), source, line);
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
