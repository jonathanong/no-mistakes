use super::*;

impl SharedTraversalContext {
    pub(crate) fn extend_check_facts(
        &mut self,
        shared: &crate::codebase::check_facts::CheckFactMap,
    ) {
        let incoming = || {
            shared
                .ts
                .iter()
                .map(|(path, facts)| (path.clone(), facts.ts.clone()))
        };
        match &mut self.facts {
            Some(facts) => facts.extend_shared(incoming()),
            None => {
                self.facts = Some(
                    crate::codebase::ts_source::facts::TsFactMap::from_shared_iter_with_plan(
                        incoming(),
                        shared.graph_plan(),
                    ),
                );
            }
        }
        self.invalidate_analysis_caches();
    }
}
