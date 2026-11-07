use super::*;

impl Bindings<'_> {
    pub(crate) fn row_member_overwritten(
        &self,
        file: &Path,
        scope: usize,
        name: &str,
        at: u32,
        members: &[String],
        call: (u32, u32),
    ) -> bool {
        if members.len() != 1
            || !self
                .overwrites
                .get(&(file, scope, name, members[0].as_str()))
                .is_some_and(|calls| calls.contains(&call))
        {
            return false;
        }
        let Some(values) = self.iterations.get(&(file, scope, name)) else {
            return false;
        };
        let index = values.partition_point(|binding| binding.declared_at <= at);
        // Loop origins remain conservative; this proof only narrows the new
        // const-array channel, never ordinary runtime value bindings.
        values
            .get(index.checked_sub(1).unwrap_or(values.len()))
            .is_some_and(|binding| !binding.repeated)
    }
    pub(crate) fn query_projection(&self, file: &Path, scope: usize, name: &str, at: u32) -> bool {
        self.row_named(file, scope, name, at, &mut crate::fx::fx_set())
    }
    fn row_named(
        &self,
        file: &Path,
        scope: usize,
        name: &str,
        at: u32,
        seen: &mut crate::fx::FxHashSet<(usize, String, u32)>,
    ) -> bool {
        let value = self.value(file, scope, name, at).or_else(|| {
            let values = self.iterations.get(&(file, scope, name))?;
            let index = values.partition_point(|value| value.declared_at <= at);
            values.get(index.checked_sub(1)?).copied()
        });
        value.is_some_and(|binding| {
            binding.invalidated_at.is_none_or(|end| at < end)
                && seen.insert((scope, name.to_owned(), binding.declared_at))
                && self.row(file, &binding.value, seen)
        })
    }
    fn row(
        &self,
        file: &Path,
        value: &ValueExpression,
        seen: &mut crate::fx::FxHashSet<(usize, String, u32)>,
    ) -> bool {
        match value {
            ValueExpression::Call(start, end) => self
                .calls
                .get(file)
                .is_some_and(|calls| calls.contains(&(*start, *end))),
            ValueExpression::Named {
                name,
                scope,
                offset,
                ..
            } => self.row_named(file, scope.unwrap_or(0), name, *offset, seen),
            ValueExpression::Derived(parts) => parts.iter().any(|part| self.row(file, part, seen)),
            ValueExpression::Either(left, right)
            | ValueExpression::Nullish(left, right)
            | ValueExpression::Default(left, right) => {
                self.row(file, left, seen) || self.row(file, right, seen)
            }
            ValueExpression::Member(base, _) | ValueExpression::Awaited(base) => {
                self.row(file, base, seen)
            }
            _ => false,
        }
    }
}
