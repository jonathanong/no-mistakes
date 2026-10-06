use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Expression, TSType, TSTypeName};
use std::collections::{HashMap, HashSet};

pub(super) struct NameHit {
    pub(super) confirmed: bool,
    pub(super) owners: Vec<u32>,
}

impl NameHit {
    fn miss() -> Self {
        Self {
            confirmed: false,
            owners: Vec::new(),
        }
    }

    fn confirmed() -> Self {
        Self {
            confirmed: true,
            owners: Vec::new(),
        }
    }

    fn provisional(owners: Vec<u32>) -> Self {
        Self {
            confirmed: false,
            owners,
        }
    }
}

pub(super) fn type_hit(
    types: &HashSet<String>,
    provisional: &HashMap<String, Vec<u32>>,
    ty: &TSType<'_>,
) -> NameHit {
    match ty {
        TSType::TSTypeReference(reference) => match &reference.type_name {
            TSTypeName::IdentifierReference(ident) => {
                name_hit(types, provisional, ident.name.as_str())
            }
            _ => NameHit::miss(),
        },
        TSType::TSUnionType(union) => union_hit(types, provisional, union),
        _ => NameHit::miss(),
    }
}

pub(super) fn factory_hit(
    factories: &HashSet<String>,
    provisional: &HashMap<String, Vec<u32>>,
    init: &Expression<'_>,
) -> NameHit {
    let mut init = unwrap_ts_wrappers(init);
    if let Expression::AwaitExpression(awaited) = init {
        init = unwrap_ts_wrappers(&awaited.argument);
    }
    let Expression::CallExpression(call) = init else {
        return NameHit::miss();
    };
    match unwrap_ts_wrappers(&call.callee) {
        Expression::Identifier(ident) => name_hit(factories, provisional, ident.name.as_str()),
        _ => NameHit::miss(),
    }
}

fn name_hit(
    confirmed: &HashSet<String>,
    provisional: &HashMap<String, Vec<u32>>,
    name: &str,
) -> NameHit {
    if confirmed.contains(name) {
        return NameHit::confirmed();
    }
    match provisional.get(name) {
        Some(owners) if !owners.is_empty() => NameHit::provisional(owners.clone()),
        _ => NameHit::miss(),
    }
}

fn union_hit(
    types: &HashSet<String>,
    provisional: &HashMap<String, Vec<u32>>,
    union: &oxc_ast::ast::TSUnionType<'_>,
) -> NameHit {
    let mut owners = Vec::new();
    for member in &union.types {
        let hit = type_hit(types, provisional, member);
        if hit.confirmed {
            return NameHit::confirmed();
        }
        for owner in hit.owners {
            if !owners.contains(&owner) {
                owners.push(owner);
            }
        }
    }
    if owners.is_empty() {
        NameHit::miss()
    } else {
        NameHit::provisional(owners)
    }
}
