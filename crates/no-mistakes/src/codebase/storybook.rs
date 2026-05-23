use crate::import_shape::is_runtime_import;
use crate::imports::collect_identifier_references;
use oxc_ast::ast::{ImportDeclarationSpecifier, Program};

#[derive(Debug, Clone, Default)]
pub(crate) struct StorybookFileFacts {
    pub(crate) used_runtime_imports: Vec<UsedRuntimeImport>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UsedRuntimeImport {
    pub(crate) source: String,
    pub(crate) imported: String,
    pub(crate) local: String,
    pub(crate) namespace: bool,
    pub(crate) line: u32,
}

pub(crate) fn extract_program(source: &str, program: &Program<'_>) -> StorybookFileFacts {
    let referenced = collect_identifier_references(program);
    let mut used_runtime_imports = Vec::new();
    for stmt in &program.body {
        let oxc_ast::ast::Statement::ImportDeclaration(import) = stmt else {
            continue;
        };
        if !is_runtime_import(import) {
            continue;
        }
        let Some(specifiers) = &import.specifiers else {
            continue;
        };
        for specifier in specifiers {
            let Some(imported) = imported_name(specifier) else {
                continue;
            };
            let local = local_name(specifier);
            if !referenced.contains(local) {
                continue;
            }
            used_runtime_imports.push(UsedRuntimeImport {
                source: import.source.value.as_str().to_string(),
                imported,
                local: local.to_string(),
                namespace: matches!(
                    specifier,
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(_)
                ),
                line: crate::codebase::ts_source::byte_offset_to_line(
                    source,
                    import.span.start as usize,
                ),
            });
        }
    }
    StorybookFileFacts {
        used_runtime_imports,
    }
}

pub(crate) fn extract_mdx_source(source: &str) -> StorybookFileFacts {
    let mut used_runtime_imports = Vec::new();
    for (index, line) in source.lines().enumerate() {
        let line_number = (index + 1) as u32;
        let trimmed = line.trim();
        let Some(import) = trimmed.strip_prefix("import ") else {
            continue;
        };
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

fn imported_name(specifier: &ImportDeclarationSpecifier<'_>) -> Option<String> {
    match specifier {
        ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => Some("default".to_string()),
        ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => Some("*".to_string()),
        ImportDeclarationSpecifier::ImportSpecifier(specifier)
            if !specifier.import_kind.is_type() =>
        {
            Some(specifier.imported.name().to_string())
        }
        ImportDeclarationSpecifier::ImportSpecifier(_) => None,
    }
}

fn local_name<'a>(specifier: &'a ImportDeclarationSpecifier<'a>) -> &'a str {
    match specifier {
        ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
            specifier.local.name.as_ref()
        }
        ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
            specifier.local.name.as_ref()
        }
        ImportDeclarationSpecifier::ImportSpecifier(specifier) => specifier.local.name.as_ref(),
    }
}
