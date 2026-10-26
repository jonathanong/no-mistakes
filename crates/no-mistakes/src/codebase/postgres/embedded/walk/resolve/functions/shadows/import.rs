use super::super::super::super::super::options::TrustedSqlTag;
use super::super::super::super::super::tags::matches_trusted_sql_import;
use super::TagShadows;
use oxc_ast::ast::{ImportDeclaration, ImportDeclarationSpecifier, ImportOrExportKind};

const TRUSTED_SQL_TAG_MODULE: &str = "sql-template-strings";

/// Record whether an import is a configured tag or a rebinding of that name.
///
/// A default import or `{ default as local }` from `sql-template-strings` is
/// the trusted tag under whatever local name the file used. A named `sql`
/// import from that module is trusted only when the local name is still
/// `sql` in any casing; a rename to a different word is not that binding.
/// A configured `trustedSqlTags` named import is trusted under its local
/// name, including a rename. A namespace import is the module object, so a
/// local `sql` there remains untrusted. Any other `sql`-spelled import is a
/// shadow: the source module is not a configured tag.
pub(super) fn record_import(
    import: &ImportDeclaration<'_>,
    shadows: &mut TagShadows,
    trusted_sql_tags: &[TrustedSqlTag],
) {
    if import.import_kind == ImportOrExportKind::Type {
        return;
    }
    let Some(specifiers) = &import.specifiers else {
        return;
    };
    let source = crate::ast::js_literal(&import.source);
    let trusted = source == TRUSTED_SQL_TAG_MODULE;
    for specifier in specifiers {
        record_specifier(specifier, trusted, &source, trusted_sql_tags, shadows);
    }
}

fn record_specifier(
    specifier: &ImportDeclarationSpecifier<'_>,
    trusted: bool,
    source: &str,
    trusted_sql_tags: &[TrustedSqlTag],
    shadows: &mut TagShadows,
) {
    match specifier {
        ImportDeclarationSpecifier::ImportSpecifier(named) => {
            if named.import_kind == ImportOrExportKind::Type {
                return;
            }
            let local = named.local.name.as_str();
            let imported = crate::ast::module_export_name(&named.imported);
            if matches_trusted_sql_import(source, imported.as_ref(), trusted_sql_tags) {
                if trusted {
                    shadows.imported.insert_template_strings(local.to_string());
                } else {
                    shadows.imported.insert_imported(local.to_string());
                }
                return;
            }
            if trusted && imported.as_ref() == "default" {
                shadows.imported.insert_template_strings(local.to_string());
                return;
            }
            if trusted && imported.as_ref() == "sql" && local.eq_ignore_ascii_case("sql") {
                shadows.imported.insert_template_strings(local.to_string());
                return;
            }
            shadow_sql_local(local, shadows);
        }
        ImportDeclarationSpecifier::ImportDefaultSpecifier(default) => {
            let local = default.local.name.as_str();
            if trusted {
                shadows.imported.insert_template_strings(local.to_string());
                return;
            }
            shadow_sql_local(local, shadows);
        }
        ImportDeclarationSpecifier::ImportNamespaceSpecifier(namespace) => {
            shadow_sql_local(namespace.local.name.as_str(), shadows);
        }
    }
}

fn shadow_sql_local(local: &str, shadows: &mut TagShadows) {
    if super::is_trusted_tag_name(local) {
        shadows.names.insert(local.to_string());
    }
}
