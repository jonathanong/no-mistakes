impl ImportCollector {
    fn collect_local_export_specifiers(&mut self, export: &ExportNamedDeclaration<'_>) {
        if !export.export_kind.is_type() {
            for specifier in &export.specifiers {
                if !specifier.export_kind.is_type() {
                    if let Some(name) = module_export_name_name(&specifier.local) {
                        self.exported_functions.insert(name.to_string());
                    }
                }
            }
        }
        self.export_depth += 1;
        walk::walk_export_named_declaration(self, export);
        self.export_depth -= 1;
    }

    fn walk_inline_export_declaration(&mut self, export: &ExportDeclaration<'_>) {
        self.export_depth += 1;
        walk::walk_export_declaration(self, export);
        self.export_depth -= 1;
    }

    fn walk_sourced_export_declaration(&mut self, export: &ExportFromDeclaration<'_>) {
        self.push_reexport(
            export.source.value.as_str(),
            export_named_declaration_kind(export),
            export.span.start as usize,
        );
        self.export_depth += 1;
        walk::walk_export_from_declaration(self, export);
        self.export_depth -= 1;
    }
}
