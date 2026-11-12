//! A const initialized by a same-file factory whose only return is a literal-headed trusted tag.
use super::{
    diagnostics::Gaps,
    helper_records::Summary,
    syntax::{Head, Syntax},
    trusted_append, HeadProof, Span,
};
use crate::codebase::ts_source::module_facts::{ModuleBinding, TypeScriptModuleFacts};
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::{Declaration, Program, Statement};

pub(super) fn register(
    syntax: &mut Syntax,
    program: &Program<'_>,
    module: Option<&TypeScriptModuleFacts>,
) {
    let Some(module) = module.filter(|_| !syntax.effect_calls.is_empty()) else {
        return;
    };
    let functions = top_level_functions(program);
    let effects = syntax
        .effect_calls
        .iter()
        .map(|call| call.builder)
        .collect::<FxHashSet<_>>();
    let imports = trusted_append::import_ids(module);
    let gaps = Gaps::new(module);
    let uses = |binding: &ModuleBinding, spans: &dyn Fn(Span) -> bool| {
        binding
            .references
            .iter()
            .any(|usage| spans((usage.span.start, usage.span.end)))
    };
    let mut roots = Vec::new();
    for (&declaration, &(callee, init)) in &syntax.record_initializers {
        if !syntax.primitive_initializers.contains(&declaration)
            || syntax.repeated_calls.contains(&init)
        {
            continue;
        }
        let name = &program.source_text[callee.0 as usize..callee.1 as usize];
        let Some(&Some(function)) = functions.get(name) else {
            continue;
        };
        let Some(&Summary::Builder {
            proof: HeadProof::Literal,
            tag,
        }) = syntax.record_returns.get(&function)
        else {
            continue;
        };
        let owned =
            |binding: &&ModuleBinding| (binding.span.start, binding.span.end) == declaration;
        let used = module.bindings.iter().find(owned).is_some_and(|binding| {
            binding.runtime && uses(binding, &|span| effects.contains(&span))
        });
        let trusted = module.bindings.iter().any(|binding| {
            binding.imported && imports.contains(&binding.id) && uses(binding, &|span| span == tag)
        });
        if used && trusted && resolves(syntax, module, &gaps, function, callee) {
            roots.push((declaration, tag));
        }
    }
    for (declaration, tag) in roots {
        syntax.constants.insert(declaration, Head::Tag);
        syntax.append_tags.insert(declaration, tag);
    }
}

fn resolves(
    syntax: &Syntax,
    module: &TypeScriptModuleFacts,
    gaps: &Gaps,
    function: Span,
    callee: Span,
) -> bool {
    module
        .bindings
        .iter()
        .find(|binding| (binding.span.start, binding.span.end) == function)
        .is_some_and(|binding| {
            binding.runtime
                && !binding.imported
                && !gaps.overlaps(function)
                && binding
                    .references
                    .iter()
                    .any(|usage| (usage.span.start, usage.span.end) == callee)
                && binding.references.iter().all(|usage| {
                    let span = (usage.span.start, usage.span.end);
                    !syntax.assignments.writes.contains_key(&span)
                        && !syntax.assignments.unsupported_writes.contains(&span)
                })
        })
}

fn top_level_functions<'a>(program: &'a Program<'_>) -> FxHashMap<&'a str, Option<Span>> {
    let mut result: FxHashMap<&str, Option<Span>> = FxHashMap::default();
    for statement in &program.body {
        let function = match statement {
            Statement::FunctionDeclaration(function) => function,
            Statement::ExportDeclaration(export) => {
                let Declaration::FunctionDeclaration(function) = &export.declaration else {
                    continue;
                };
                function
            }
            _ => continue,
        };
        let Some(id) = &function.id else {
            continue;
        };
        result
            .entry(id.name.as_str())
            .and_modify(|span| *span = None)
            .or_insert(Some((id.span.start, id.span.end)));
    }
    result
}
