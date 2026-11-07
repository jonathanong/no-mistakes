impl ImportCollector {
    fn predeclare_bounded_statement(&mut self, statement: &Statement<'_>) {
        if !self.collect_per_item_calls {
            return;
        }
        match statement {
            Statement::VariableDeclaration(declaration) => {
                self.predeclare_bounded_variables(declaration)
            }
            Statement::TSTypeAliasDeclaration(declaration) => {
                self.predeclare_bounded_type(declaration)
            }
            Statement::TSEnumDeclaration(declaration) => self.record_bounded_enum(declaration),
            Statement::ExportDeclaration(export) => match &export.declaration {
                Declaration::VariableDeclaration(declaration) => {
                    self.predeclare_bounded_variables(declaration)
                }
                Declaration::TSTypeAliasDeclaration(declaration) => {
                    self.predeclare_bounded_type(declaration)
                }
                Declaration::TSEnumDeclaration(declaration) => {
                    self.record_bounded_enum(declaration)
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn predeclare_bounded_variables(&mut self, declaration: &VariableDeclaration<'_>) {
        if declaration.kind != VariableDeclarationKind::Const {
            return;
        }
        for declarator in &declaration.declarations {
            let Some(name) = binding_identifier_name(&declarator.id) else {
                continue;
            };
            let Some(init) = &declarator.init else {
                continue;
            };
            let value = if literal_const_tuple(init) {
                Some(BoundedValue::Tuple)
            } else if literal_fixed_tuple(init) {
                Some(BoundedValue::Sequence)
            } else if predeclared_number(init) {
                self.bounded_number(init).map(BoundedValue::Number)
            } else {
                None
            };
            if let Some(value) = value {
                self.bounded_domains
                    .values
                    .entry(self.current_lexical_scope_id())
                    .or_default()
                    .entry(name.to_string())
                    .or_insert(value);
            }
        }
    }

    fn predeclare_bounded_type(&mut self, declaration: &TSTypeAliasDeclaration<'_>) {
        if predeclared_literal_domain(&declaration.type_annotation) {
            self.record_bounded_type(declaration);
        }
    }
}

fn predeclared_literal_domain(ty: &oxc_ast::ast::TSType<'_>) -> bool {
    use oxc_ast::ast::{TSLiteral, TSType};
    match ty {
        TSType::TSLiteralType(literal) => matches!(
            literal.literal,
            TSLiteral::StringLiteral(_) | TSLiteral::NumericLiteral(_)
        ),
        TSType::TSUnionType(union) => {
            !union.types.is_empty() && union.types.iter().all(predeclared_literal_domain)
        }
        TSType::TSParenthesizedType(inner) => predeclared_literal_domain(&inner.type_annotation),
        _ => false,
    }
}

fn predeclared_number(expression: &Expression<'_>) -> bool {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
        Expression::NumericLiteral(_) => true,
        Expression::UnaryExpression(unary)
            if unary.operator == oxc_ast::ast::UnaryOperator::UnaryNegation =>
        {
            predeclared_number(&unary.argument)
        }
        _ => false,
    }
}
