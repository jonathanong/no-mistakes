use super::*;
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::config::v2::schema::EffectTargetConfig;
use oxc_ast::ast::{TSType, TSTypeName};

mod freeze;
mod parameters;
mod patterns;
mod possible;
mod runtime;
mod transports;
mod visit;

// Match both type references and parameters by semantic identity. A local type
// with the same spelling as the configured import is never an executor proof.
pub(super) fn collect(
    program: &Program<'_>,
    module: &TypeScriptModuleFacts,
    profile: &EmbeddedSqlOptions,
    selection: &Selection,
    declared_types: &[EffectTargetConfig],
) -> (
    FxHashSet<usize>,
    FxHashSet<usize>,
    FxHashSet<Span>,
    FxHashSet<Span>,
) {
    let declared = declared_types
        .iter()
        .map(|selector| (selector.module.as_str(), selector.export.as_str()))
        .collect::<FxHashSet<_>>();
    let declared = &declared;
    let types = module
        .imports
        .iter()
        .flat_map(|import| {
            import
                .bindings
                .iter()
                .filter(move |binding| {
                    profile.executor_type_names.contains(&binding.imported)
                        && (super::super::embedded::scoped_bindings::from_configured_module(
                            &import.specifier,
                            &profile.import_specifier,
                        ) || declared
                            .contains(&(import.specifier.as_str(), binding.imported.as_str())))
                })
                .map(|binding| binding.binding_id)
        })
        .collect::<FxHashSet<_>>();
    let references = module
        .bindings
        .iter()
        .filter(|binding| types.contains(&binding.id))
        .flat_map(|binding| &binding.references)
        .map(|reference| (reference.span.start, reference.span.end))
        .collect();
    let bindings = module
        .bindings
        .iter()
        .flat_map(|binding| {
            std::iter::once(((binding.span.start, binding.span.end), binding.id)).chain(
                binding
                    .references
                    .iter()
                    .map(|usage| ((usage.span.start, usage.span.end), binding.id)),
            )
        })
        .collect();
    let candidates = types.union(&selection.unsupported).copied().collect();
    let possible = possible::references(program, module, &candidates, &bindings);
    let mut visitor = Typed {
        references,
        freeze: freeze::Tracker::new(
            module.diagnostics.is_empty(),
            !selection.selected.is_empty()
                || !selection.unsupported.is_empty()
                || !selection.namespaces.is_empty()
                || !selection.factories.is_empty()
                || !types.is_empty(),
        ),
        possible,
        bindings,
        selected: FxHashSet::default(),
        unsupported: FxHashSet::default(),
        sources: transports::sources(module, selection),
        transports: FxHashSet::default(),
        factory_bindings: FxHashMap::default(),
        factory_arguments: Vec::new(),
        writes: FxHashSet::default(),
        immutable: false,
    };
    visitor.visit_program(program);
    let mut facts = runtime::collect(program, &visitor.possible, &visitor.bindings);
    visitor.transports.extend(facts.transports.iter().copied());
    visitor.selected.extend(facts.values.iter().copied());
    visitor.unsupported.extend(facts.values.iter().copied());
    visitor.finish_transports();
    let initial = selection
        .selected
        .iter()
        .chain(&selection.namespaces)
        .chain(&selection.unsupported)
        .chain(&visitor.selected)
        .copied()
        .collect();
    let producers = facts
        .functions
        .union(&selection.factories)
        .copied()
        .collect();
    let proven = visitor
        .selected
        .difference(&visitor.unsupported)
        .copied()
        .collect();
    let sources = freeze::Sources {
        initial: &initial,
        proven: &proven,
        factories: &selection.factories,
        writes: &visitor.writes,
        candidates: &facts.uses,
        data: &facts.data,
        producers: &producers,
        handled: &visitor.transports,
    };
    let proof = std::mem::take(&mut visitor.freeze).finish(&sources);
    visitor.selected.extend(proof.bindings.iter().copied());
    visitor.unsupported.extend(proof.bindings);
    facts.uses.extend(facts.data.iter().copied());
    facts.uses.extend(proof.runtime);
    visitor.transports.extend(proof.transports);
    visitor.finish_factory_arguments();
    (
        visitor.selected,
        visitor.unsupported,
        facts.uses,
        visitor.transports,
    )
}

struct Typed {
    freeze: freeze::Tracker,
    references: FxHashSet<Span>,
    possible: FxHashSet<Span>,
    bindings: FxHashMap<Span, usize>,
    selected: FxHashSet<usize>,
    unsupported: FxHashSet<usize>,
    sources: FxHashMap<Span, bool>,
    transports: FxHashSet<Span>,
    factory_bindings: FxHashMap<usize, bool>,
    factory_arguments: Vec<Span>,
    writes: FxHashSet<usize>,
    immutable: bool,
}

impl Typed {
    fn direct_type(&self, ty: &TSType<'_>) -> bool {
        matches!(ty, TSType::TSTypeReference(reference)
            if matches!(&reference.type_name, TSTypeName::IdentifierReference(name)
                if self.references.contains(&span(name.span))))
    }
    // Only direct imported types on formal parameters prove literal SQL. Other
    // annotation shapes retain runtime Unknown uses without evaluating SQL.
    fn record(&mut self, binding: &oxc_ast::ast::BindingIdentifier<'_>, ty: &TSType<'_>) {
        if let Some(id) = self.bindings.get(&span(binding.span)) {
            let direct = self.direct_type(ty);
            if direct || possible::contains(&self.possible, ty) {
                self.selected.insert(*id);
                if !direct {
                    self.unsupported.insert(*id);
                }
            }
        }
    }
}
