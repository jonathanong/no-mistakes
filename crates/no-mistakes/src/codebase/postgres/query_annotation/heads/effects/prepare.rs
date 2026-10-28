use super::{CallShape, CallSyntax, Signature, Span};
use crate::codebase::postgres::query_annotation::heads::{
    bindings::Bindings,
    syntax::{Head, Syntax},
};
use crate::codebase::ts_source::module_facts::{ModuleBinding, TypeScriptModuleFacts};
use crate::fx::{FxHashMap, FxHashSet};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub(crate) struct Pending {
    pub span: Span,
    pub builder: Span,
    pub specifier: String,
    pub imported: String,
    pub shape: CallShape,
}

#[derive(Default)]
pub(in crate::codebase::postgres::query_annotation::heads) struct Prepared {
    pub(crate) allowed: FxHashSet<Span>,
    pub(crate) pending: Vec<Pending>,
    pub(crate) exported: BTreeMap<String, Signature>,
}

pub(in crate::codebase::postgres::query_annotation::heads) fn prepare(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    module: &TypeScriptModuleFacts,
    trusted_roots: &FxHashSet<Span>,
) -> Prepared {
    let mut declarations: FxHashMap<Span, Option<&ModuleBinding>> = FxHashMap::default();
    for binding in &module.bindings {
        declarations
            .entry((binding.span.start, binding.span.end))
            .and_modify(|owner| *owner = None)
            .or_insert(Some(binding));
    }
    let proven = super::summary::proven(syntax, bindings, &declarations);
    let imports = imports(module);
    let mut result = Prepared::default();
    for call in &syntax.effect_calls {
        classify_call(
            call,
            syntax,
            bindings,
            trusted_roots,
            &proven,
            &imports,
            &mut result,
        );
    }
    result.exported = exported(module, &proven);
    result.pending.sort_unstable_by_key(|value| value.span);
    result
}

fn classify_call(
    call: &CallSyntax,
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    trusted_roots: &FxHashSet<Span>,
    proven: &FxHashMap<(usize, u8), Signature>,
    imports: &FxHashMap<usize, Option<(&str, &str)>>,
    out: &mut Prepared,
) {
    let Some(builder) = bindings.owner(call.builder).filter(|owner| !owner.imported) else {
        return;
    };
    let declaration = (builder.span.start, builder.span.end);
    if !trusted_roots.contains(&declaration)
        || !matches!(
            syntax.constants.get(&declaration),
            Some(Head::Tag | Head::AppendTag | Head::AppendPrefixTag)
        )
    {
        return;
    }
    let Some(callee) = bindings.owner(call.callee) else {
        return;
    };
    if callee.imported {
        if call.position != 0 {
            return;
        }
        let Some((specifier, imported)) = imports.get(&callee.id).copied().flatten() else {
            return;
        };
        out.pending.push(Pending {
            span: call.span,
            builder: call.builder,
            specifier: specifier.to_string(),
            imported: imported.to_string(),
            shape: call.shape,
        });
    } else if proven
        .get(&(callee.id, call.position))
        .is_some_and(|signature| signature.accepts(call.shape))
    {
        out.allowed.insert(call.builder);
    }
}

fn imports(module: &TypeScriptModuleFacts) -> FxHashMap<usize, Option<(&str, &str)>> {
    let mut result = FxHashMap::default();
    for import in module.imports.iter().filter(|import| !import.type_only) {
        for binding in import
            .bindings
            .iter()
            .filter(|binding| !binding.type_only && binding.imported != "*")
        {
            result
                .entry(binding.binding_id)
                .and_modify(|value| *value = None)
                .or_insert(Some((import.specifier.as_str(), binding.imported.as_str())));
        }
    }
    result
}

fn exported(
    module: &TypeScriptModuleFacts,
    proven: &FxHashMap<(usize, u8), Signature>,
) -> BTreeMap<String, Signature> {
    let roots = module
        .scopes
        .iter()
        .filter(|scope| scope.parent_id.is_none())
        .map(|scope| scope.id)
        .collect::<FxHashSet<_>>();
    let mut locals: FxHashMap<&str, Option<usize>> = FxHashMap::default();
    for binding in module
        .bindings
        .iter()
        .filter(|b| b.runtime && !b.imported && roots.contains(&b.scope_id))
    {
        locals
            .entry(binding.name.as_str())
            .and_modify(|owner| *owner = None)
            .or_insert(Some(binding.id));
    }
    let mut seen = FxHashSet::default();
    let mut ambiguous = FxHashSet::default();
    let mut result = BTreeMap::new();
    for export in module
        .exports
        .iter()
        .filter(|e| !e.type_only && e.specifier.is_none())
    {
        if !seen.insert(export.exported.as_str()) {
            result.remove(export.exported.as_str());
            ambiguous.insert(export.exported.as_str());
        }
        if ambiguous.contains(export.exported.as_str()) {
            continue;
        }
        let Some(signature) = locals
            .get(export.local.as_str())
            .copied()
            .flatten()
            .and_then(|id| proven.get(&(id, 0)).copied())
        else {
            continue;
        };
        result.insert(export.exported.clone(), signature);
    }
    result
}
