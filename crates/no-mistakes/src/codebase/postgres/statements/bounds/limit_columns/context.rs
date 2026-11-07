use super::{work, Relations};
use crate::codebase::postgres::statements::value::PlaceholderPositions;
use sqlparser::ast::TableFactor;

pub(in crate::codebase::postgres::statements::bounds) enum RelationContext<'a> {
    Pending {
        inherited: Option<&'a Relations>,
        factors: Vec<&'a TableFactor>,
    },
    Ready(Relations),
}

impl<'a> RelationContext<'a> {
    pub(in crate::codebase::postgres::statements::bounds) fn new(inherited: &'a Relations) -> Self {
        Self::Pending {
            inherited: Some(inherited),
            factors: Vec::new(),
        }
    }

    pub(in crate::codebase::postgres::statements::bounds) fn add(
        &mut self,
        factor: &'a TableFactor,
        positions: PlaceholderPositions<'_>,
    ) {
        match self {
            Self::Pending { factors, .. } => factors.push(factor),
            Self::Ready(relations) => {
                work::builder_relation_add();
                relations.add(factor, positions);
            }
        }
    }

    pub(in crate::codebase::postgres::statements::bounds) fn reset(&mut self) {
        match self {
            Self::Pending { inherited, factors } => {
                *inherited = None;
                factors.clear();
            }
            Self::Ready(relations) => *relations = Relations::default(),
        }
    }

    pub(in crate::codebase::postgres::statements::bounds) fn visible(
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
                    // Builder uses one immutable positions slice for recording and replay.
                    for factor in factors.drain(..) {
                        work::builder_relation_add();
                        relations.add(factor, positions);
                    }
                    *self = Self::Ready(relations);
                }
            }
        }
    }
}
