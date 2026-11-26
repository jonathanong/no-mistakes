use super::{literal_glob, SpreadImport};
use crate::ast::{js_literal, module_export_name};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    BindingPattern, Declaration, Expression, ImportDeclarationSpecifier, Program, Statement,
};
use oxc_ast::AstKind;
use oxc_semantic::{Semantic, SymbolId};
use rustc_hash::{FxHashMap, FxHashSet};

pub(super) enum Spread {
    Literal(FxHashSet<String>),
    Imported(SpreadImport),
    Unknown,
}

pub(crate) fn has_candidate_exports(program: &Program<'_>) -> bool {
    export_arrays(program).next().is_some()
}

pub(super) fn exports(
    program: &Program<'_>,
    semantic: &Semantic<'_>,
) -> FxHashMap<String, FxHashSet<String>> {
    export_arrays(program)
        .filter_map(|(name, init)| {
            let symbol = name.symbol_id.get()?;
            if !safe_references(semantic, symbol) {
                return None;
            }
            Some((name.name.to_string(), literal_array(init)?))
        })
        .collect()
}

fn export_arrays<'a>(
    program: &'a Program<'a>,
) -> impl Iterator<Item = (&'a oxc_ast::ast::BindingIdentifier<'a>, &'a Expression<'a>)> {
    program
        .body
        .iter()
        .filter_map(|statement| {
            let Statement::ExportDeclaration(export) = statement else {
                return None;
            };
            let Declaration::VariableDeclaration(declaration) = &export.declaration else {
                return None;
            };
            declaration.kind.is_const().then_some(declaration)
        })
        .flat_map(|declaration| &declaration.declarations)
        .filter_map(|declaration| {
            let BindingPattern::BindingIdentifier(name) = &declaration.id else {
                return None;
            };
            let expr = declaration.init.as_ref()?;
            matches!(unwrap_ts_wrappers(expr), Expression::ArrayExpression(_))
                .then_some((&**name, expr))
        })
}

pub(super) fn relative_imports(program: &Program<'_>) -> FxHashMap<SymbolId, SpreadImport> {
    let mut imports = FxHashMap::default();
    for statement in &program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        let specifier = js_literal(&import.source);
        if import.import_kind.is_type()
            || !(specifier.starts_with("./") || specifier.starts_with("../"))
        {
            continue;
        }
        for binding in import
            .specifiers
            .iter()
            .flatten()
            .filter_map(|binding| match binding {
                ImportDeclarationSpecifier::ImportSpecifier(binding)
                    if !binding.import_kind.is_type() =>
                {
                    Some(binding)
                }
                _ => None,
            })
        {
            if let Some(symbol) = binding.local.symbol_id.get() {
                imports.insert(
                    symbol,
                    SpreadImport {
                        specifier: specifier.to_string(),
                        name: module_export_name(&binding.imported).into_owned(),
                    },
                );
            }
        }
    }
    imports
}

pub(super) fn read(
    expression: &Expression<'_>,
    semantic: &Semantic<'_>,
    imports: &FxHashMap<SymbolId, SpreadImport>,
) -> Spread {
    if let Some(items) = literal_array(expression) {
        return Spread::Literal(items);
    }
    let Expression::Identifier(identifier) = unwrap_ts_wrappers(expression) else {
        return Spread::Unknown;
    };
    let scoping = semantic.scoping();
    let Some(symbol) = identifier
        .reference_id
        .get()
        .and_then(|id| scoping.get_reference(id).symbol_id())
        .filter(|&symbol| safe_references(semantic, symbol))
    else {
        return Spread::Unknown;
    };
    if let Some(import) = imports.get(&symbol) {
        return Spread::Imported(import.clone());
    }
    let declaration = semantic.nodes().kind(scoping.symbol_declaration(symbol));
    if scoping.symbol_flags(symbol).is_const_variable() {
        if let Some(declaration) = declaration.as_variable_declarator() {
            if declaration.span.end <= identifier.span.start
                && declaration.id.is_binding_identifier()
            {
                if let Some(items) = declaration.init.as_ref().and_then(literal_array) {
                    return Spread::Literal(items);
                }
            }
        }
    }
    Spread::Unknown
}

fn safe_references(semantic: &Semantic<'_>, symbol: SymbolId) -> bool {
    let scope = semantic.scoping();
    scope.symbol_redeclarations(symbol).is_empty()
        && scope.get_resolved_references(symbol).all(|reference| {
            reference.is_type()
                || (!reference.is_write()
                    && matches!(
                        semantic.nodes().parent_kind(reference.node_id()),
                        AstKind::SpreadElement(_)
                    ))
        })
}

fn literal_array(expression: &Expression<'_>) -> Option<FxHashSet<String>> {
    let Expression::ArrayExpression(array) = unwrap_ts_wrappers(expression) else {
        return None;
    };
    array
        .elements
        .iter()
        .map(|element| literal_glob(element.as_expression()?).map(|glob| glob.into_owned()))
        .collect()
}
