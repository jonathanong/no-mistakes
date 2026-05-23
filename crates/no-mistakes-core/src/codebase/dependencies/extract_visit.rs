#[derive(Default)]
struct ImportCollector {
    imports: Vec<ExtractedImport>,
    function_calls: Vec<FunctionCall>,
    unknown_callers: Vec<Option<String>>,
    function_stack: Vec<String>,
    exported_functions: HashSet<String>,
    export_depth: usize,
    has_unknown_top_level_call: bool,
}

impl ImportCollector {
    fn push(&mut self, specifier: &str, kind: ImportKind) {
        if !specifier.is_empty() {
            self.imports.push(ExtractedImport {
                specifier: specifier.to_string(),
                kind,
                function_scope: self.function_stack.last().cloned(),
            });
        }
    }

    fn push_function_scope(&mut self, name: Option<String>) {
        if let Some(name) = name {
            let scope = self
                .function_stack
                .last()
                .map(|parent| format!("{parent}/{name}"))
                .unwrap_or(name);
            if self.export_depth > 0 && self.function_stack.is_empty() {
                self.exported_functions.insert(scope.clone());
            }
            self.function_stack.push(scope);
        }
    }

    fn pop_function_scope(&mut self, pushed: bool) {
        if pushed {
            self.function_stack.pop();
        }
    }

    fn current_function(&self) -> Option<String> {
        self.function_stack.last().cloned()
    }
}

impl<'a> Visit<'a> for ImportCollector {
    fn visit_function(
        &mut self,
        function: &oxc::ast::ast::Function<'a>,
        flags: oxc_syntax::scope::ScopeFlags,
    ) {
        let name = function_name(function);
        let pushed = name.is_some();
        self.push_function_scope(name);
        walk::walk_function(self, function, flags);
        self.pop_function_scope(pushed);
    }

    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        let name = binding_identifier_name(&declarator.id).map(str::to_string);
        match declarator.init.as_ref() {
            Some(Expression::ArrowFunctionExpression(arrow)) => {
                walk::walk_binding_pattern(self, &declarator.id);
                let pushed = name.is_some();
                self.push_function_scope(name);
                walk::walk_arrow_function_expression(self, arrow);
                self.pop_function_scope(pushed);
            }
            Some(Expression::FunctionExpression(function)) => {
                walk::walk_binding_pattern(self, &declarator.id);
                let scope_name = match name {
                    Some(name) => Some(name),
                    None => function_name(function),
                };
                let pushed = scope_name.is_some();
                self.push_function_scope(scope_name);
                walk::walk_function(self, function, oxc_syntax::scope::ScopeFlags::empty());
                self.pop_function_scope(pushed);
            }
            _ => walk::walk_variable_declarator(self, declarator),
        }
    }

    fn visit_import_declaration(&mut self, import: &ImportDeclaration<'a>) {
        let kind = import_declaration_kind(import);
        self.push(import.source.value.as_str(), kind);
    }

    fn visit_export_named_declaration(&mut self, export: &ExportNamedDeclaration<'a>) {
        if let Some(source) = &export.source {
            let kind = export_named_declaration_kind(export);
            self.push(source.value.as_str(), kind);
        } else if !export.export_kind.is_type() {
            for specifier in &export.specifiers {
                if specifier.export_kind.is_type() {
                    continue;
                }
                if let Some(name) = module_export_name_name(&specifier.local) {
                    self.exported_functions.insert(name.to_string());
                }
            }
        }
        self.export_depth += 1;
        walk::walk_export_named_declaration(self, export);
        self.export_depth -= 1;
    }

    fn visit_export_all_declaration(&mut self, export: &ExportAllDeclaration<'a>) {
        let kind = if export.export_kind.is_type() {
            ImportKind::Type
        } else {
            ImportKind::Static
        };
        self.push(export.source.value.as_str(), kind);
    }

    fn visit_export_default_declaration(&mut self, export: &ExportDefaultDeclaration<'a>) {
        if let ExportDefaultDeclarationKind::Identifier(identifier) = &export.declaration {
            self.exported_functions.insert(identifier.name.to_string());
        }
        self.export_depth += 1;
        walk::walk_export_default_declaration(self, export);
        self.export_depth -= 1;
    }

    fn visit_import_expression(&mut self, import: &ImportExpression<'a>) {
        if let Some(specifier) = string_literal_expr(&import.source) {
            self.push(specifier, ImportKind::Dynamic);
        }
        walk::walk_import_expression(self, import);
    }

    fn visit_ts_import_type(&mut self, import: &TSImportType<'a>) {
        self.push(import.source.value.as_str(), ImportKind::Type);
        walk::walk_ts_import_type(self, import);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if is_require_callee(&call.callee) {
            if let Some(first) = call.arguments.first() {
                if let Some(specifier) = string_literal_argument(first) {
                    self.push(specifier, ImportKind::Require);
                }
            }
        } else if let Some(callee) = simple_callee_name(&call.callee) {
            self.function_calls.push(FunctionCall {
                caller: self.current_function(),
                callee: callee.to_string(),
            });
        } else {
            let caller = self.current_function();
            if caller.is_none() {
                self.has_unknown_top_level_call = true;
            }
            self.unknown_callers.push(caller);
        }
        walk::walk_call_expression(self, call);
    }
}

fn function_name(function: &oxc::ast::ast::Function<'_>) -> Option<String> {
    let id = function.id.as_ref()?;
    Some(id.name.to_string())
}
