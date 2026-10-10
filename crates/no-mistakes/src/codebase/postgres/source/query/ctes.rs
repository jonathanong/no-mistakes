use super::*;
impl Collector<'_, '_> {
    pub(super) fn ctes(
        &mut self,
        query: &Query,
        scope: usize,
        outer: &CteEnvironment,
    ) -> CteEnvironment {
        let mut env = outer.clone();
        let Some(with) = &query.with else {
            return env;
        };
        let first = self.facts.ctes.len();
        for cte in &with.cte_tables {
            let id = self.facts.ctes.len();
            self.facts.ctes.push(PostgresSqlQueryCte {
                id,
                owner_scope_id: scope,
                query_scope_id: 0,
                name: identifier(&cte.alias.name),
                column_aliases: cte
                    .alias
                    .columns
                    .iter()
                    .map(|c| identifier(&c.name))
                    .collect(),
                recursive: with.recursive,
                referenced: false,
                used: false,
                cyclic: false,
                span: self.locations.span(cte.span()),
            });
            if with.recursive {
                env.insert(identifier(&cte.alias.name).identity, id);
            }
        }
        for (index, cte) in with.cte_tables.iter().enumerate() {
            let id = first + index;
            let child = self.query(
                &cte.query,
                Some(scope),
                self.states[scope].visible_parent,
                PostgresSqlQueryClause::Cte,
                &env,
                Some(id),
            );
            let locations = self.locations;
            super::statement_bounds::repair(
                &mut self.facts,
                locations,
                child,
                &cte.closing_paren_token,
            );
            self.facts.ctes[id].query_scope_id = child;
            env.insert(identifier(&cte.alias.name).identity, id);
        }
        env
    }
    pub(super) fn finish_ctes(&mut self) {
        let mut edges = vec![BTreeSet::new(); self.facts.ctes.len()];
        let mut used = BTreeSet::new();
        for relation in &self.facts.relations {
            if let Some(target) = relation.cte_id {
                self.facts.ctes[target].referenced = true;
                if let Some(owner) = self.facts.scopes[relation.scope_id].cte_definition_id {
                    edges[owner].insert(target);
                } else {
                    used.insert(target);
                }
            }
        }
        let mut pending: Vec<_> = used.iter().copied().collect();
        while let Some(id) = pending.pop() {
            for &next in &edges[id] {
                if used.insert(next) {
                    pending.push(next);
                }
            }
        }
        for cte in &mut self.facts.ctes {
            cte.used = used.contains(&cte.id);
            let mut visited = BTreeSet::new();
            let mut pending: Vec<_> = edges[cte.id].iter().copied().collect();
            while let Some(id) = pending.pop() {
                if id == cte.id {
                    cte.cyclic = true;
                    break;
                }
                if visited.insert(id) {
                    pending.extend(edges[id].iter().copied());
                }
            }
        }
    }
}
