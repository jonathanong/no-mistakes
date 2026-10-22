use super::*;
use oxc_ast::ast::{ImportDeclarationSpecifier, Statement};

/// Recovery syntax certifies selection only, never a positive SQL proof.
pub(crate) fn collect_invalid(
    program: &Program<'_>,
    source: &str,
    profiles: &[EmbeddedSqlOptions],
) -> FileFacts {
    let mut output = FileFacts::default();
    for profile in profiles {
        let mut calls = Vec::new();
        if profile.selects_executors() {
            for statement in &program.body {
                let Statement::ImportDeclaration(import) = statement else {
                    continue;
                };
                let exact = profile.import_specifier.is_empty()
                    || import.source.value.as_str() == profile.import_specifier;
                let scoped = exact
                    || import
                        .source
                        .value
                        .starts_with(&format!("{}/", profile.import_specifier));
                if !scoped {
                    continue;
                }
                let selected = import
                    .specifiers
                    .iter()
                    .flatten()
                    .any(|binding| match binding {
                        ImportDeclarationSpecifier::ImportSpecifier(named) => {
                            let name = named.imported.name();
                            (exact
                                && !import.import_kind.is_type()
                                && !named.import_kind.is_type()
                                && profile
                                    .executor_names
                                    .iter()
                                    .any(|target| target == name.as_str()))
                                || profile
                                    .executor_factory_names
                                    .iter()
                                    .chain(&profile.executor_type_names)
                                    .any(|target| target == name.as_str())
                        }
                        ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {
                            !profile.import_specifier.is_empty() && !import.import_kind.is_type()
                        }
                        _ => false,
                    });
                if selected {
                    let at = span(import.span);
                    calls.push(SqlCall {
                        span: at,
                        line: crate::codebase::ts_source::byte_offset_to_line(
                            source,
                            at.0 as usize,
                        ),
                        input: Input::Unknown,
                    });
                }
            }
        }
        output.profiles.push((profile.clone(), calls));
    }
    output
}
