use super::super::super::super::super::options::TrustedSqlTag;
use super::super::super::super::super::tags::matches_trusted_sql_import;
use super::TagShadows;
use oxc_ast::ast::{ImportDeclaration, ImportDeclarationSpecifier, ImportOrExportKind};

const TRUSTED_SQL_TAG_MODULE: &str = "sql-template-strings";

/// Record whether an import rebinds `sql` away from the trusted tag.
///
/// A default import (or `import { default as local }`) from
/// `sql-template-strings` *is* the trusted tag: store the local name so
/// classification can trust it even when it isn't spelled `sql`. A named
/// `sql` import from that module is not a shadow either; trust still comes
/// from the `sql` spelling. A configured `trustedSqlTags` named import is
/// the same kind of opt-in: store its local name, including a rename.
/// A namespace import is always the module object, so a local `sql` there
/// remains untrusted. Any other `sql`-spelled import is a shadow: the
/// source module is not the real tag library.
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
    let source = import.source.value.as_str();
    let trusted = source == TRUSTED_SQL_TAG_MODULE;
    for specifier in specifiers {
        record_specifier(specifier, trusted, source, trusted_sql_tags, shadows);
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
            let imported = named.imported.name();
            if matches_trusted_sql_import(source, imported.as_str(), trusted_sql_tags) {
                shadows.imported.insert_imported(local.to_string());
                return;
            }
            if trusted && imported.as_str() == "default" {
                shadows.imported.insert_imported(local.to_string());
                return;
            }
            if trusted && imported.as_str() == "sql" && local.eq_ignore_ascii_case("sql") {
                return;
            }
            shadow_sql_local(local, shadows);
        }
        ImportDeclarationSpecifier::ImportDefaultSpecifier(default) => {
            let local = default.local.name.as_str();
            if trusted {
                shadows.imported.insert_imported(local.to_string());
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
