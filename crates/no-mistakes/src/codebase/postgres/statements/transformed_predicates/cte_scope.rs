use crate::codebase::postgres::idents::ident_key;
use crate::fx::FxHashMap;
use sqlparser::ast::Query;

#[derive(Default)]
pub(super) struct QueryScopes(Vec<Frame>);
struct Frame {
    visible: Vec<String>,
    cte_inputs: FxHashMap<usize, Vec<String>>,
}
impl QueryScopes {
    pub(super) fn enter(&mut self, query: &Query) {
        let inherited = self
            .0
            .last()
            .map(|frame| {
                frame
                    .cte_inputs
                    .get(&(query as *const Query as usize))
                    .unwrap_or(&frame.visible)
                    .clone()
            })
            .unwrap_or_default();
        let own = query
            .with
            .iter()
            .flat_map(|with| &with.cte_tables)
            .map(|cte| ident_key(&cte.alias.name))
            .collect::<Vec<_>>();
        let mut visible = inherited.clone();
        visible.extend(own.iter().cloned());
        let mut cte_inputs = FxHashMap::default();
        if let Some(with) = &query.with {
            for (index, cte) in with.cte_tables.iter().enumerate() {
                // Nonrecursive bodies see only earlier CTEs; their own name can still be a base table.
                let scope = if with.recursive {
                    visible.clone()
                } else {
                    inherited.iter().chain(&own[..index]).cloned().collect()
                };
                cte_inputs.insert(cte.query.as_ref() as *const Query as usize, scope);
            }
        }
        self.0.push(Frame {
            visible,
            cte_inputs,
        });
    }
    pub(super) fn exit(&mut self) {
        self.0.pop();
    }
    pub(super) fn visible(&self) -> &[String] {
        self.0.last().map_or(&[], |frame| frame.visible.as_slice())
    }
}
