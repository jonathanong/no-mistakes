macro_rules! visit_type_and_expression_methods {
    () => {
        fn visit_ts_type_alias_declaration(&mut self, declaration: &TSTypeAliasDeclaration<'a>) {
            visit_ts_type_alias_declaration_with_scope(self, declaration);
        }

        fn visit_ts_interface_declaration(&mut self, declaration: &TSInterfaceDeclaration<'a>) {
            visit_ts_interface_declaration_with_scope(self, declaration);
        }

        fn visit_ts_enum_declaration(&mut self, declaration: &TSEnumDeclaration<'a>) {
            visit_ts_enum_declaration_with_scope(self, declaration);
        }

        fn visit_ts_module_block(&mut self, block: &TSModuleBlock<'a>) {
            visit_ts_module_block_with_depth(self, block);
        }

        fn visit_ts_namespace_declaration(&mut self, namespace: &TSNamespaceDeclaration<'a>) {
            visit_ts_namespace_declaration_with_path(self, namespace);
        }

        fn visit_binary_expression(&mut self, binary: &BinaryExpression<'a>) {
            visit_binary_expression_with_namespaces(self, binary);
        }

        fn visit_import_declaration(&mut self, import: &ImportDeclaration<'a>) {
            visit_import_declaration_with_scope(self, import);
        }

        fn visit_export_named_declaration(&mut self, export: &ExportNamedDeclaration<'a>) {
            self.collect_local_export_specifiers(export);
        }

        fn visit_export_declaration(&mut self, export: &ExportDeclaration<'a>) {
            self.walk_inline_export_declaration(export);
        }

        fn visit_export_from_declaration(&mut self, export: &ExportFromDeclaration<'a>) {
            self.walk_sourced_export_declaration(export);
        }

        fn visit_export_all_declaration(&mut self, export: &ExportAllDeclaration<'a>) {
            visit_export_all_declaration_with_scope(self, export);
        }

        fn visit_export_default_declaration(&mut self, export: &ExportDefaultDeclaration<'a>) {
            self.record_default_value(export);
            visit_export_default_declaration_with_scope(self, export);
        }

        fn visit_import_expression(&mut self, import: &ImportExpression<'a>) {
            visit_import_expression_with_scope(self, import);
        }

        fn visit_ts_import_type(&mut self, import: &TSImportType<'a>) {
            visit_ts_import_type_with_scope(self, import);
        }

        fn visit_return_statement(&mut self, statement: &oxc_ast::ast::ReturnStatement<'a>) {
            self.visit_value_return(statement);
        }

        fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
            self.visit_receiver_call(call);
        }

        fn visit_await_expression(&mut self, expression: &oxc_ast::ast::AwaitExpression<'a>) {
            visit_await_expression_with_capture(self, expression);
        }

        fn visit_yield_expression(&mut self, expression: &oxc_ast::ast::YieldExpression<'a>) {
            visit_yield_expression_with_capture(self, expression);
        }

        fn visit_unary_expression(&mut self, expression: &oxc_ast::ast::UnaryExpression<'a>) {
            visit_unary_expression_with_delete(self, expression);
        }

        fn visit_new_expression(&mut self, new: &NewExpression<'a>) {
            visit_new_expression_with_calls(self, new);
        }

        fn visit_tagged_template_expression(&mut self, tagged: &TaggedTemplateExpression<'a>) {
            visit_tagged_template_with_receiver(self, tagged);
        }
    };
}
