use super::{CallShape, CallSyntax, Signature, Span};
use crate::codebase::postgres::query_annotation::heads::{
    bindings::Bindings,
    syntax::{Head, Syntax},
};
use crate::codebase::ts_source::module_facts::{ModuleBinding, TypeScriptModuleFacts};
use crate::fx::{FxHashMap, FxHashSet};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub(crate) struct Pending {
    pub span: Span,
    pub builder: Span,
    pub specifier: String,
    pub imported: String,
    pub shape: CallShape,
    pub require_plain: bool,
}

impl Pending {
    pub(crate) fn proven_by(&self, heads: &super::super::AnnotationHeads) -> bool {
        (!self.require_plain || heads.exported_closed_effects.contains(&self.imported))
            && heads
                .exported_effects
                .get(&self.imported)
                .is_some_and(|signature| self.accepts(*signature))
    }
    pub(crate) fn accepts(&self, signature: Signature) -> bool {
        (!self.require_plain || signature.plain()) && signature.accepts(self.shape)
    }
}

#[derive(Default)]
pub(in crate::codebase::postgres::query_annotation::heads) struct Prepared {
    pub(crate) allowed: FxHashSet<Span>,
    pub(crate) pending: Vec<Pending>,
    pub(crate) exported: BTreeMap<String, Signature>,
    pub(crate) closed_exports: BTreeSet<String>,
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
    let imports = imports(module);
    let summaries = super::summary::proven(syntax, bindings, &declarations, &imports);
    let mut result = Prepared::default();
    for call in &syntax.effect_calls {
        classify_call(
            call,
            syntax,
            bindings,
            trusted_roots,
            &summaries,
            &imports,
            &mut result,
        );
    }
    (result.exported, result.closed_exports) = super::exports::collect(module, &summaries);
    result.pending.sort_unstable_by_key(|value| value.span);
    result
}

fn classify_call(
    call: &CallSyntax,
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    trusted_roots: &FxHashSet<Span>,
    proven: &FxHashMap<(usize, u8), super::summary::Summary>,
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
            require_plain: false,
        });
    } else if let Some(summary) = proven.get(&(callee.id, call.position)) {
        if !summary.signature.accepts(call.shape) {
            return;
        }
        if let Some(required) = &summary.required {
            let mut pending = required.clone();
            pending.span = call.span;
            pending.builder = call.builder;
            out.pending.push(pending);
        } else {
            out.allowed.insert(call.builder);
        }
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
