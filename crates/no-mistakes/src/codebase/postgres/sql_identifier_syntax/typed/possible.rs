use super::*;
use oxc_ast::ast::{
    IdentifierReference, TSInterfaceDeclaration, TSTypeAliasDeclaration, TSTypeParameter,
};

// Alias reachability is only a negative proof: it retains runtime unknowns,
// never promotes an indirect annotation into a literal executor proof.
pub(super) fn references(
    program: &Program<'_>,
    module: &TypeScriptModuleFacts,
    types: &FxHashSet<usize>,
    bindings: &FxHashMap<Span, usize>,
) -> FxHashSet<Span> {
    if types.is_empty() {
        return FxHashSet::default();
    }
    let owners = module
        .bindings
        .iter()
        .flat_map(|binding| {
            binding
                .references
                .iter()
                .map(move |reference| ((reference.span.start, reference.span.end), binding.id))
        })
        .collect::<FxHashMap<_, _>>();
    let mut aliases = Aliases {
        bindings,
        owners: &owners,
        readers: FxHashMap::default(),
    };
    aliases.visit_program(program);
    let mut selected = types.clone();
    let mut pending = types.iter().copied().collect::<Vec<_>>();
    while let Some(id) = pending.pop() {
        for alias in aliases.readers.get(&id).into_iter().flatten() {
            if selected.insert(*alias) {
                pending.push(*alias);
            }
        }
    }
    owners
        .into_iter()
        .filter_map(|(at, id)| selected.contains(&id).then_some(at))
        .collect()
}

struct Aliases<'a> {
    bindings: &'a FxHashMap<Span, usize>,
    owners: &'a FxHashMap<Span, usize>,
    readers: FxHashMap<usize, Vec<usize>>,
}
impl<'a> Visit<'a> for Aliases<'_> {
    fn visit_ts_type_parameter(&mut self, parameter: &TSTypeParameter<'a>) {
        if let Some(id) = self.bindings.get(&span(parameter.name.span)) {
            let mut uses = Uses {
                owners: self.owners,
                found: FxHashSet::default(),
            };
            uses.visit_ts_type_parameter(parameter);
            for used in uses.found {
                self.readers.entry(used).or_default().push(*id);
            }
        }
        walk::walk_ts_type_parameter(self, parameter);
    }
    fn visit_ts_interface_declaration(&mut self, interface: &TSInterfaceDeclaration<'a>) {
        if let Some(id) = self.bindings.get(&span(interface.id.span)) {
            let mut uses = Uses {
                owners: self.owners,
                found: FxHashSet::default(),
            };
            for heritage in &interface.extends {
                uses.visit_ts_interface_heritage(heritage);
            }
            uses.visit_ts_interface_body(&interface.body);
            for used in uses.found {
                self.readers.entry(used).or_default().push(*id);
            }
        }
        walk::walk_ts_interface_declaration(self, interface);
    }
    fn visit_ts_type_alias_declaration(&mut self, alias: &TSTypeAliasDeclaration<'a>) {
        if let Some(id) = self.bindings.get(&span(alias.id.span)) {
            let mut uses = Uses {
                owners: self.owners,
                found: FxHashSet::default(),
            };
            uses.visit_ts_type(&alias.type_annotation);
            for used in uses.found {
                self.readers.entry(used).or_default().push(*id);
            }
        }
        walk::walk_ts_type_alias_declaration(self, alias);
    }
}
struct Uses<'a> {
    owners: &'a FxHashMap<Span, usize>,
    found: FxHashSet<usize>,
}
impl<'a> Visit<'a> for Uses<'_> {
    fn visit_identifier_reference(&mut self, reference: &IdentifierReference<'a>) {
        if let Some(id) = self.owners.get(&span(reference.span)) {
            self.found.insert(*id);
        }
    }
}

pub(super) fn contains(references: &FxHashSet<Span>, ty: &TSType<'_>) -> bool {
    struct Contains<'a> {
        references: &'a FxHashSet<Span>,
        found: bool,
    }
    impl<'a> Visit<'a> for Contains<'_> {
        fn visit_identifier_reference(&mut self, reference: &IdentifierReference<'a>) {
            self.found |= self.references.contains(&span(reference.span));
        }
    }
    let mut visitor = Contains {
        references,
        found: false,
    };
    visitor.visit_ts_type(ty);
    visitor.found
}
