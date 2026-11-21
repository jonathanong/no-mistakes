impl ImportCollector {
    fn record_local_export_binding(
        &mut self,
        local: impl Into<String>,
        exported: impl Into<String>,
    ) {
        let binding = ExportedBinding {
            specifier: None,
            local: local.into(),
            exported: exported.into(),
        };
        if !self.call_export_bindings.contains(&binding) {
            self.call_export_bindings.push(binding);
        }
    }

    fn note_export_name_offset(&mut self, local: &str, offset: u32) {
        self.namespace
            .facts
            .export_name_offsets
            .push((local.to_string(), offset));
    }

    fn note_sourced_export_offset(
        &mut self,
        specifier: &str,
        local: &str,
        exported: &str,
        offset: u32,
    ) {
        self.namespace.facts.sourced_export_offsets.push((
            specifier.to_string(),
            local.to_string(),
            exported.to_string(),
            offset,
        ));
    }

    fn collect_local_export_specifiers(&mut self, export: &ExportNamedDeclaration<'_>) {
        self.mark_export_clause_locals(export);
        if self.collect_value_flow {
            for specifier in &export.specifiers {
                self.value_flow.type_exports.push(ExportedBinding {
                    specifier: None,
                    local: crate::ast::module_export_name(&specifier.local).into_owned(),
                    exported: crate::ast::module_export_name(&specifier.exported).into_owned(),
                });
            }
        }
        if !export.export_kind.is_type() {
            for specifier in &export.specifiers {
                if !specifier.export_kind.is_type() {
                    if let (Some(local), Some(exported)) = (
                        module_export_name_name(&specifier.local),
                        module_export_name_name(&specifier.exported),
                    ) {
                        self.note_exported_registry(local.as_ref());
                        self.closed_registry_reads
                            .insert(specifier.local.span().start);
                        self.note_export_name_offset(local.as_ref(), export.span.start);
                        self.record_local_export_binding(local, exported);
                    }
                    if let ModuleExportName::IdentifierName(identifier) = &specifier.local {
                        self.exported_functions.insert(identifier.name.to_string());
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
        if self.collect_value_flow {
            for specifier in &export.specifiers {
                self.value_flow.type_exports.push(ExportedBinding {
                    specifier: Some(crate::ast::js_literal(&export.source).into_owned()),
                    local: crate::ast::module_export_name(&specifier.local).into_owned(),
                    exported: crate::ast::module_export_name(&specifier.exported).into_owned(),
                });
            }
        }
        self.push_reexport(
            crate::ast::js_literal(&export.source).as_ref(),
            export_named_declaration_kind(export),
            export.span.start as usize,
        );
        if !export.export_kind.is_type() {
            for specifier in &export.specifiers {
                if !specifier.export_kind.is_type() {
                    if let (Some(local), Some(exported)) = (
                        module_export_name_name(&specifier.local),
                        module_export_name_name(&specifier.exported),
                    ) {
                        let source = crate::ast::js_literal(&export.source);
                        self.note_sourced_export_offset(
                            source.as_ref(),
                            local.as_ref(),
                            exported.as_ref(),
                            export.span.start,
                        );
                        self.call_export_bindings.push(ExportedBinding {
                            specifier: Some(source.into_owned()),
                            local: local.to_string(),
                            exported: exported.to_string(),
                        });
                    }
                }
            }
        }
        self.export_depth += 1;
        walk::walk_export_from_declaration(self, export);
        self.export_depth -= 1;
    }
}
