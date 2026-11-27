use super::TypeScriptModuleFacts;
use oxc_ast::AstKind;
use oxc_semantic::Semantic;
use oxc_span::Span;
use oxc_syntax::symbol::SymbolId;

pub(super) fn overload(semantic: &Semantic<'_>, id: SymbolId) -> bool {
    let scoping = semantic.scoping();
    let declarations = std::iter::once(scoping.symbol_declaration(id)).chain(
        scoping
            .symbol_redeclarations(id)
            .iter()
            .map(|item| item.declaration),
    );
    let mut implementations = 0;
    for declaration in declarations {
        let AstKind::Function(function) = semantic.nodes().kind(declaration) else {
            return false;
        };
        implementations += usize::from(function.body.is_some());
    }
    implementations == 1
}

pub(super) fn ambient(kind: AstKind<'_>, semantic: &Semantic<'_>, spans: &mut Vec<Span>) {
    let definition = semantic.source_type().is_typescript_definition();
    match kind {
        AstKind::TSExternalModuleDeclaration(value) if definition || value.declare => {
            spans.push(value.span);
        }
        AstKind::TSNamespaceDeclaration(value) if definition || value.declare => {
            spans.push(value.span);
        }
        AstKind::TSGlobalDeclaration(value) => spans.push(value.span),
        AstKind::TSExportAssignment(value) if definition => spans.push(value.span),
        _ => {}
    }
}

pub(super) fn finish_ambient(
    spans: &[Span],
    references: &[u32],
    facts: &mut TypeScriptModuleFacts,
) {
    if spans.is_empty() {
        return;
    }
    let mut ranges: Vec<_> = spans
        .iter()
        .copied()
        .filter(|span| {
            let at = references.partition_point(|at| *at < span.start);
            references.get(at).is_none_or(|at| *at >= span.end)
        })
        .collect();
    ranges.sort_unstable_by_key(|span| (span.start, std::cmp::Reverse(span.end)));
    let mut merged: Vec<Span> = Vec::new();
    for span in ranges {
        if let Some(last) = merged.last_mut().filter(|last| last.end >= span.start) {
            last.end = last.end.max(span.end);
        } else {
            merged.push(span);
        }
    }
    for diagnostic in &facts.diagnostics {
        if diagnostic.kind != "unsupported"
            || !matches!(
                diagnostic.message.as_str(),
                "Unsupported module or dynamic binding form"
                    | "Merged binding declarations are unsupported"
            )
        {
            continue;
        }
        let Some(inner) = diagnostic.span else {
            continue;
        };
        let at = merged.partition_point(|span| span.start <= inner.start);
        if at > 0 && inner.end <= merged[at - 1].end {
            facts.importer_safe.push((inner.start, inner.end));
        }
    }
}
