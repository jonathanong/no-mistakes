fn binding_identifier_name<'a>(pattern: &'a oxc_ast::ast::BindingPattern<'a>) -> Option<&'a str> {
    match pattern {
        oxc_ast::ast::BindingPattern::BindingIdentifier(identifier) => {
            Some(identifier.name.as_str())
        }
        _ => None,
    }
}

fn simple_callee_name(expr: &Expression<'_>) -> Option<String> {
    match expr {
        Expression::Identifier(ident) => Some(ident.name.to_string()),
        Expression::ParenthesizedExpression(parenthesized) => {
            simple_callee_name(&parenthesized.expression)
        }
        Expression::StaticMemberExpression(member) => simple_static_member_name(member),
        _ => None,
    }
}

fn simple_static_member_name(member: &StaticMemberExpression<'_>) -> Option<String> {
    match &member.object {
        Expression::Identifier(object) => Some(format!(
            "{}.{}",
            object.name.as_str(),
            member.property.name.as_str()
        )),
        _ => None,
    }
}

fn jsx_element_reference_name(name: &oxc_ast::ast::JSXElementName<'_>) -> Option<String> {
    match name {
        oxc_ast::ast::JSXElementName::Identifier(id) => Some(id.name.to_string()),
        oxc_ast::ast::JSXElementName::IdentifierReference(id) => Some(id.name.to_string()),
        oxc_ast::ast::JSXElementName::MemberExpression(member) => {
            jsx_member_expression_name(member)
        }
        _ => None,
    }
}

fn jsx_member_expression_name(member: &oxc_ast::ast::JSXMemberExpression<'_>) -> Option<String> {
    let object = match &member.object {
        oxc_ast::ast::JSXMemberExpressionObject::IdentifierReference(id) => id.name.to_string(),
        oxc_ast::ast::JSXMemberExpressionObject::MemberExpression(member) => {
            jsx_member_expression_name(member)?
        }
        oxc_ast::ast::JSXMemberExpressionObject::ThisExpression(_) => return None,
    };
    Some(format!("{}.{}", object, member.property.name.as_str()))
}

fn type_reference_name(reference: &TSTypeReference<'_>) -> Option<String> {
    ts_type_name_name(&reference.type_name)
}

fn ts_type_name_name(name: &TSTypeName<'_>) -> Option<String> {
    match name {
        TSTypeName::IdentifierReference(identifier) => Some(identifier.name.to_string()),
        TSTypeName::QualifiedName(qualified) => ts_qualified_name_name(qualified),
        TSTypeName::ThisExpression(_) => None,
    }
}

fn ts_qualified_name_name(name: &TSQualifiedName<'_>) -> Option<String> {
    let left = ts_type_name_name(&name.left)?;
    Some(format!("{}.{}", left, name.right.name.as_str()))
}

fn import_declaration_kind(import: &ImportDeclaration<'_>) -> ImportKind {
    if import.import_kind.is_type()
        || all_named_specifiers_are_type(import.specifiers.as_deref().map(|v| &**v))
    {
        ImportKind::Type
    } else {
        ImportKind::Static
    }
}

fn export_named_declaration_kind(export: &ExportNamedDeclaration<'_>) -> ImportKind {
    if export.export_kind.is_type() || all_export_specifiers_are_type(&export.specifiers) {
        ImportKind::Type
    } else {
        ImportKind::Static
    }
}

fn all_named_specifiers_are_type(specifiers: Option<&[ImportDeclarationSpecifier<'_>]>) -> bool {
    let Some(specifiers) = specifiers else {
        return false;
    };
    !specifiers.is_empty()
        && specifiers.iter().all(|spec| {
            matches!(
                spec,
                ImportDeclarationSpecifier::ImportSpecifier(s) if s.import_kind.is_type()
            )
        })
}

fn all_export_specifiers_are_type(specifiers: &[ExportSpecifier<'_>]) -> bool {
    !specifiers.is_empty() && specifiers.iter().all(|s| s.export_kind.is_type())
}

fn module_export_name_name<'a>(name: &'a ModuleExportName<'a>) -> Option<&'a str> {
    if let ModuleExportName::IdentifierReference(identifier) = name {
        Some(identifier.name.as_str())
    } else {
        None
    }
}

fn is_require_callee(expr: &Expression<'_>) -> bool {
    matches!(expr, Expression::Identifier(ident) if ident.name == "require")
}

fn is_require_resolve_callee(expr: &Expression<'_>) -> bool {
    matches!(expr, Expression::StaticMemberExpression(member)
        if matches!(&member.object, Expression::Identifier(ident) if ident.name == "require")
            && member.property.name == "resolve")
}

fn static_import_specifier(expr: &Expression<'_>) -> Option<String> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(s) => Some(s.value.as_str().to_string()),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            let mut specifier = String::new();
            for quasi in &template.quasis {
                specifier.push_str(quasi.value.cooked.as_ref().unwrap_or(&quasi.value.raw));
            }
            Some(specifier)
        }
        _ => None,
    }
}

fn string_literal_argument<'a>(arg: &'a Argument<'a>) -> Option<&'a str> {
    match arg {
        Argument::StringLiteral(s) => Some(s.value.as_str()),
        _ => None,
    }
}

fn static_path_argument(arg: &Argument<'_>) -> Option<String> {
    match arg {
        Argument::StringLiteral(s) => Some(s.value.as_str().to_string()),
        Argument::TemplateLiteral(tl) if tl.expressions.is_empty() => {
            Some(crate::codebase::ts_routes::refs::normalize_template(tl))
        }
        _ => None,
    }
}

fn static_process_cwd_arg(callee: &str, args: &[Argument<'_>]) -> Option<String> {
    let method = callee.rsplit('.').next().unwrap_or(callee);
    let opts_index = match method {
        "exec" => 1,
        "execFile" | "fork" | "spawn" => 2,
        _ => return None,
    };
    crate::codebase::ts_process_spawn::extract_cwd_from_opts(args, opts_index)
}
