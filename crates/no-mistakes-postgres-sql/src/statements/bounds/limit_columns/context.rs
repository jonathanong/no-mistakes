use super::{columns::relation_key, work, Relations};
use crate::statements::value::PlaceholderPositions;
use sqlparser::ast::TableFactor;

pub(in crate::statements::bounds) enum RelationContext<'a> {
    Pending {
        inherited: Option<&'a Relations>,
        factors: Vec<(&'a TableFactor, Option<String>, bool)>,
    },
    Ready(Relations),
}

impl<'a> RelationContext<'a> {
    pub(in crate::statements::bounds) fn new(inherited: &'a Relations) -> Self {
        Self::Pending {
            inherited: Some(inherited),
            factors: Vec::new(),
        }
    }

    pub(in crate::statements::bounds) fn add(
        &mut self,
        factor: &'a TableFactor,
        positions: PlaceholderPositions<'_>,
    ) -> Option<String> {
        let key = relation_key(factor);
        match self {
            Self::Pending { factors, .. } => factors.push((factor, key.clone(), false)),
            Self::Ready(relations) => {
                work::builder_relation_add();
                relations.add(factor, positions);
            }
        }
        key
    }

    pub(in crate::statements::bounds) fn null_extend_keys(&mut self, keys: &[String]) {
        match self {
            Self::Pending { factors, .. } => {
                for (_, key, null_extended) in factors.iter_mut() {
                    if key.as_ref().is_some_and(|key| keys.contains(key)) {
                        *null_extended = true;
                    }
                }
            }
            Self::Ready(relations) => relations.null_extend(keys),
        }
    }

    pub(in crate::statements::bounds) fn visible(
        &mut self,
        positions: PlaceholderPositions<'_>,
    ) -> &Relations {
        loop {
            match self {
                Self::Ready(relations) => return relations,
                Self::Pending { inherited, factors } => {
                    work::builder_materialization();
                    let mut relations =
                        (*inherited).map_or_else(Relations::default, |parent| parent.clone());
                    relations.begin_from();
                    for (factor, key, null_extended) in factors.drain(..) {
                        work::builder_relation_add();
                        relations.add(factor, positions);
                        if null_extended {
                            if let Some(key) = &key {
                                relations.null_extend(std::slice::from_ref(key));
                            }
                        }
                    }
                    *self = Self::Ready(relations);
                }
            }
        }
    }
}
