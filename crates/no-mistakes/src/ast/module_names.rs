//! Keep Oxc 0.153's fatal rejection of spelling-sensitive surrogate names.
use super::ParserReturn;
use oxc_allocator::{Allocator, Dummy, Vec};
use oxc_ast::ast::{
    Declaration, ImportDeclarationSpecifier, ModuleExportName, Program, Statement,
    TSNamespaceDeclarationBody,
};
use oxc_span::Span;

pub(super) fn enforce<'a>(
    mut parsed: ParserReturn<'a>,
    allocator: &'a Allocator,
) -> ParserReturn<'a> {
    let span = if parsed.fatal_error {
        record_names::first_legacy_name(&parsed.module_record, parsed.program.source_text).or_else(
            || {
                if !parsed.program.source_type.is_typescript() {
                    return None;
                }
                let limit = parsed
                    .diagnostics
                    .iter()
                    .filter_map(|diagnostic| diagnostic.labels.first())
                    .map(|label| label.offset() as usize)
                    .max()?;
                prefix_names::first_legacy_name(parsed.program.source_text, limit)
            },
        )
    } else {
        parsed.program.body.iter().find_map(statement_name)
    };
    let Some(span) = span else {
        return parsed;
    };
    // The old lexer had already read the next token, including its trivia.
    let mut trivia_end = span.end as usize;
    for comment in parsed
        .program
        .comments
        .iter()
        .filter(|comment| comment.span.start >= span.end)
    {
        if !parsed.program.source_text[trivia_end..comment.span.start as usize]
            .chars()
            .all(is_trivia_space)
        {
            break;
        }
        trivia_end = comment.span.end as usize;
    }
    trivia_end += parsed.program.source_text[trivia_end..]
        .chars()
        .take_while(|character| is_trivia_space(*character))
        .map(char::len_utf8)
        .sum::<usize>();
    parsed
        .program
        .comments
        .retain(|comment| comment.span.start < trivia_end as u32);
    finalization::diagnostics(&mut parsed, span, trivia_end);
    let mut empty = Program::dummy(allocator);
    empty.source_type = parsed.program.source_type;
    empty.source_text = parsed.program.source_text;
    empty.comments = std::mem::replace(&mut parsed.program.comments, Vec::new_in(&allocator));
    parsed.program = empty;
    parsed.tokens.clear();
    parsed.irregular_whitespaces = parsed
        .irregular_whitespaces
        .iter()
        .copied()
        .filter(|span| span.start < trivia_end as u32)
        .collect();
    parsed.fatal_error = true;
    parsed
}

fn is_trivia_space(character: char) -> bool {
    (character.is_whitespace() && character != '\u{85}') || character == '\u{feff}'
}

fn name_span(name: &ModuleExportName<'_>) -> Option<Span> {
    match name {
        ModuleExportName::StringLiteral(literal)
            if super::js_literal_has_lone_surrogate(literal) =>
        {
            Some(literal.span)
        }
        _ => None,
    }
}

fn statement_name(statement: &Statement<'_>) -> Option<Span> {
    match statement {
        Statement::ImportDeclaration(import) => {
            import
                .specifiers
                .iter()
                .flatten()
                .find_map(|specifier| match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                        name_span(&specifier.imported)
                    }
                    _ => None,
                })
        }
        Statement::ExportNamedDeclaration(export) => {
            export.specifiers.iter().find_map(|specifier| {
                name_span(&specifier.local).or_else(|| name_span(&specifier.exported))
            })
        }
        Statement::ExportFromDeclaration(export) => {
            export.specifiers.iter().find_map(|specifier| {
                name_span(&specifier.local).or_else(|| name_span(&specifier.exported))
            })
        }
        Statement::ExportDeclaration(export) => declaration_name(&export.declaration),
        Statement::ExportAllDeclaration(export) => export.exported.as_ref().and_then(name_span),
        _ => statement.as_declaration().and_then(declaration_name),
    }
}

fn declaration_name(declaration: &Declaration<'_>) -> Option<Span> {
    match declaration {
        Declaration::TSNamespaceDeclaration(namespace) => namespace_name(&namespace.body),
        Declaration::TSExternalModuleDeclaration(module) => module
            .body
            .as_ref()
            .and_then(|body| body.body.iter().find_map(statement_name)),
        Declaration::TSGlobalDeclaration(global) => {
            global.body.body.iter().find_map(statement_name)
        }
        _ => None,
    }
}

fn namespace_name(body: &TSNamespaceDeclarationBody<'_>) -> Option<Span> {
    match body {
        TSNamespaceDeclarationBody::TSNamespaceDeclaration(namespace) => {
            namespace_name(&namespace.body)
        }
        TSNamespaceDeclarationBody::TSModuleBlock(block) => {
            block.body.iter().find_map(statement_name)
        }
    }
}

mod finalization;
mod prefix_names;
mod record_names;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod finalization_tests;
