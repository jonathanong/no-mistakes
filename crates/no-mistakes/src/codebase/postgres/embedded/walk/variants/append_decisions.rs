use super::super::ScopeVisitor;
use super::Recovered;

impl ScopeVisitor<'_> {
    /// Carry actual guard off assignments through free composition helpers.
    pub(in crate::codebase::postgres::embedded::walk::variants) fn mark_append(
        &self,
        values: &mut [Recovered],
        site: u32,
    ) {
        let offs = self.append_decision_offs.borrow();
        for value in values {
            value.enumerated = true;
            if !value.append_sites.contains(&site) {
                value.append_sites.push(site);
            }
            for path in &self.variant_paths {
                if value.choices.iter().any(|(id, arm)| {
                    path.iter()
                        .any(|(other, branch)| id == other && arm != branch)
                }) {
                    continue;
                }
                value.decision_offs.extend(
                    path.iter().filter_map(|(id, _)| {
                        offs.get(id).map(|(source, arm)| (*id, *source, *arm))
                    }),
                );
            }
        }
    }
}

pub(super) fn same_sql(left: &Recovered, right: &Recovered) -> bool {
    left.value == right.value
        && left.sql == right.sql
        && left.line == right.line
        && left.origins == right.origins
        && left.positions == right.positions
        && left.fragment == right.fragment
}

pub(super) fn conditional(visitor: &ScopeVisitor<'_>, value: &mut Recovered, id: u64) {
    if let Some((source, arm)) = visitor.append_decision_offs.borrow().get(&id) {
        value.decision_offs.push((id, *source, *arm));
    }
}
