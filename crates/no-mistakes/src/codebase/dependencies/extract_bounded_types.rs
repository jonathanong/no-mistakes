impl ImportCollector {
    fn closed_domain_type(&self, ty: &oxc_ast::ast::TSType<'_>) -> bool {
        use oxc_ast::ast::{TSLiteral, TSType, TSTypeQueryExprName};
        match ty {
            TSType::TSLiteralType(literal) => matches!(
                literal.literal,
                TSLiteral::StringLiteral(_) | TSLiteral::NumericLiteral(_)
            ),
            TSType::TSUnionType(union) => {
                !union.types.is_empty() && union.types.iter().all(|ty| self.closed_domain_type(ty))
            }
            TSType::TSParenthesizedType(inner) => self.closed_domain_type(&inner.type_annotation),
            TSType::TSTypeReference(reference) if reference.type_arguments.is_none() => {
                let name = type_reference_name(reference).unwrap_or_default();
                if self.type_parameter_shadows(&name) {
                    return false;
                }
                for (depth, locals) in self.type_local_stack.iter().enumerate().rev() {
                    if locals.contains(&name) {
                        return self
                            .lexical_scope_ids
                            .get(depth)
                            .and_then(|scope| self.bounded_domains.types.get(scope))
                            .is_some_and(|names| names.contains(&name));
                    }
                }
                false
            }
            TSType::TSIndexedAccessType(indexed)
                if matches!(indexed.index_type, TSType::TSNumberKeyword(_)) =>
            {
                let TSType::TSTypeQuery(query) = &indexed.object_type else {
                    return false;
                };
                let TSTypeQueryExprName::IdentifierReference(id) = &query.expr_name else {
                    return false;
                };
                matches!(
                    self.bounded_value(id.name.as_str()),
                    Some(BoundedValue::Tuple)
                )
            }
            _ => false,
        }
    }

    fn closed_record(&self, ty: &oxc_ast::ast::TSType<'_>) -> bool {
        let oxc_ast::ast::TSType::TSTypeReference(reference) = ty else {
            return false;
        };
        if type_reference_name(reference).as_deref() != Some("Record")
            || self.type_binding_shadows("Record")
            || self.type_parameter_shadows("Record")
            || self.bounded_domains.imported_record
        {
            return false;
        }
        reference
            .type_arguments
            .as_ref()
            .is_some_and(|args| args.params.len() == 2 && self.closed_domain_type(&args.params[0]))
    }

    fn record_bounded_type(&mut self, declaration: &TSTypeAliasDeclaration<'_>) {
        if self.collect_per_item_calls
            && declaration.type_parameters.is_none()
            && self.closed_domain_type(&declaration.type_annotation)
        {
            self.bounded_domains
                .types
                .entry(self.current_lexical_scope_id())
                .or_default()
                .insert(declaration.id.name.to_string());
        }
    }

    fn record_bounded_enum(&mut self, declaration: &TSEnumDeclaration<'_>) {
        if !self.collect_per_item_calls {
            return;
        }
        let scope = self.current_lexical_scope_id();
        self.bounded_domains
            .types
            .entry(scope)
            .or_default()
            .insert(declaration.id.name.to_string());
        self.bounded_domains
            .values
            .entry(scope)
            .or_default()
            .entry(declaration.id.name.to_string())
            .or_insert(BoundedValue::Enum);
    }
}

impl ImportCollector {
    fn builtin_bounded_object(&self) -> bool {
        !self.bounded_domains.imported_object && self.callee_binding_scope("Object").is_none()
    }

    fn record_bounded_import(&mut self, import: &ImportDeclaration<'_>) {
        if !self.collect_per_item_calls {
            return;
        }
        for specifier in import.specifiers.iter().flatten() {
            let (name, type_only) = match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(specifier) => (
                    specifier.local.name.as_str(),
                    import.import_kind.is_type() || specifier.import_kind.is_type(),
                ),
                ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                    (specifier.local.name.as_str(), import.import_kind.is_type())
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                    (specifier.local.name.as_str(), import.import_kind.is_type())
                }
            };
            self.bounded_domains.imported_record |= name == "Record";
            self.bounded_domains.imported_object |= name == "Object" && !type_only;
        }
    }
}
