use super::*;
use oxc_syntax::{node::NodeId, symbol::SymbolId};

/// Export names a `typeof import("x")` type provably selects, or `None` for a
/// whole-module type. `Pick<typeof import("x"), "a" | "b">` selects `a` and
/// `b`. A type argument of the `importOriginal` parameter, or an `as`
/// assertion, inside the factory of a Vitest mock of that same module selects
/// nothing: the assertion is erased and the factory's runtime handling is
/// unchanged.
pub(super) fn selected(
    value: &TSImportType<'_>,
    node: NodeId,
    semantic: &Semantic<'_>,
) -> Option<Vec<String>> {
    if value.qualifier.is_some() || value.type_arguments.is_some() {
        return None;
    }
    let nodes = semantic.nodes();
    let query = nodes.parent_id(node);
    let AstKind::TSTypeQuery(_) = nodes.kind(query) else {
        return None;
    };
    let specifier = crate::ast::js_literal(&value.source);
    match nodes.parent_kind(query) {
        AstKind::TSTypeParameterInstantiation(arguments) => {
            let list = nodes.parent_id(query);
            picked(arguments, nodes.kind(query).span(), list, semantic)
                .or_else(|| importer_argument(list, &specifier, semantic))
        }
        AstKind::TSAsExpression(_) => factory(query, &specifier, semantic).map(|_| Vec::new()),
        _ => None,
    }
}

fn picked(
    arguments: &TSTypeParameterInstantiation<'_>,
    query: oxc_span::Span,
    list: NodeId,
    semantic: &Semantic<'_>,
) -> Option<Vec<String>> {
    let AstKind::TSTypeReference(reference) = semantic.nodes().parent_kind(list) else {
        return None;
    };
    let TSTypeName::IdentifierReference(name) = &reference.type_name else {
        return None;
    };
    let global = name.name == "Pick"
        && name
            .reference_id
            .get()
            .is_some_and(|id| semantic.scoping().get_reference(id).symbol_id().is_none());
    let [first, keys] = arguments.params.as_slice() else {
        return None;
    };
    if global && first.span() == query {
        literal_keys(keys)
    } else {
        None
    }
}

fn literal_keys(key: &TSType<'_>) -> Option<Vec<String>> {
    match key {
        TSType::TSLiteralType(literal) => match &literal.literal {
            TSLiteral::StringLiteral(value) => {
                Some(vec![crate::ast::js_literal(value).into_owned()])
            }
            _ => None,
        },
        TSType::TSUnionType(union) => {
            let mut names = Vec::new();
            for member in &union.types {
                names.extend(literal_keys(member)?);
            }
            Some(names)
        }
        _ => None,
    }
}

fn importer_argument(
    list: NodeId,
    specifier: &str,
    semantic: &Semantic<'_>,
) -> Option<Vec<String>> {
    let AstKind::CallExpression(call) = semantic.nodes().parent_kind(list) else {
        return None;
    };
    let Expression::Identifier(callee) = call.callee.get_inner_expression() else {
        return None;
    };
    let symbol = semantic
        .scoping()
        .get_reference(callee.reference_id.get()?)
        .symbol_id()?;
    (factory(list, specifier, semantic)? == Some(symbol)).then(Vec::new)
}

/// The first-parameter symbol of the nearest enclosing function that is the
/// factory of `vi.mock("<specifier>", factory)`.
fn factory(node: NodeId, specifier: &str, semantic: &Semantic<'_>) -> Option<Option<SymbolId>> {
    let nodes = semantic.nodes();
    nodes.ancestors(node).find_map(|ancestor| {
        let (span, params) = match ancestor.kind() {
            AstKind::ArrowFunctionExpression(function) => (function.span, &function.params),
            AstKind::Function(function) => (function.span, &function.params),
            _ => return None,
        };
        let AstKind::CallExpression(call) = nodes.parent_kind(ancestor.id()) else {
            return None;
        };
        let [Argument::StringLiteral(path), factory] = call.arguments.as_slice() else {
            return None;
        };
        if !mocks::vitest_mock_call(call, semantic)
            || path.value != specifier
            || factory.span() != span
        {
            return None;
        }
        Some(match params.items.first().map(|item| &item.pattern) {
            Some(BindingPattern::BindingIdentifier(id)) => id.symbol_id.get(),
            _ => None,
        })
    })
}
