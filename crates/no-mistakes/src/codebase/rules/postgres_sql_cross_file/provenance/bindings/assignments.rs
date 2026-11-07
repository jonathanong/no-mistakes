use super::flow::Seen;
use super::*;

impl Bindings<'_> {
    pub(super) fn assignment_unbounded(
        &self,
        file: &Path,
        scope: usize,
        name: &str,
        at: u32,
        seen: &mut Seen,
    ) -> Option<bool> {
        if !self.numeric_parameter_bounds_only {
            return None;
        }
        let writes = self.assignments.get(&(file, scope, name))?;
        let end = writes.partition_point(|write| write.completed_at <= at);
        let values = self.values.get(&(file, scope, name));
        let binding = values.and_then(|values| {
            let index = values.partition_point(|binding| binding.declared_at <= at);
            index
                .checked_sub(1)
                .and_then(|index| values.get(index))
                .copied()
        });
        let declared_at = binding.map_or(0, |binding| binding.declared_at);
        let mut found = false;
        for write in writes[..end]
            .iter()
            .rev()
            .take_while(|write| write.completed_at >= declared_at)
        {
            found = true;
            let key = (scope, name.to_string(), write.completed_at);
            if !seen.bindings.insert(key.clone()) {
                return Some(false);
            }
            let unbounded = self.expr_unbounded(file, &write.value, seen);
            seen.bindings.remove(&key);
            if unbounded {
                return Some(true);
            }
            if write.unconditional && binding.is_some_and(|binding| binding.owner == write.owner) {
                return Some(false);
            }
        }
        found.then(|| {
            binding.map_or_else(
                || self.parameter(file, scope, name),
                |binding| self.expr_unbounded(file, &binding.value, seen),
            )
        })
    }
}
