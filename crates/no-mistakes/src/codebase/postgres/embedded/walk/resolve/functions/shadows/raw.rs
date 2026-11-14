use super::*;
use oxc_ast::ast::ImportDeclarationSpecifier;
impl TagShadows {
    pub(in crate::codebase::postgres::embedded::walk::resolve::functions) fn configure_raw(
        &mut self,
        program: &Program<'_>,
        tags: &[TrustedSqlTag],
        builder_types: &HashSet<String>,
    ) {
        if tags.is_empty() {
            return;
        }
        for statement in &program.body {
            let Statement::ImportDeclaration(import) = statement else {
                continue;
            };
            if import.import_kind.is_type() {
                continue;
            }
            let module = crate::ast::js_literal(&import.source);
            for specifier in import.specifiers.iter().flatten() {
                let ImportDeclarationSpecifier::ImportSpecifier(named) = specifier else {
                    continue;
                };
                if named.import_kind.is_type() {
                    continue;
                }
                let name = crate::ast::module_export_name(&named.imported);
                if crate::codebase::postgres::embedded::tags::matches_trusted_sql_import(
                    &module, &name, tags,
                ) {
                    self.names.remove(named.local.name.as_str());
                    self.imported.insert_raw(named.local.name.to_string());
                }
            }
        }
        self.imported.collect_fragments(program, builder_types);
    }
}
