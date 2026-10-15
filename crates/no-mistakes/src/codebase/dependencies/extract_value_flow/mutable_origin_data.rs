use super::*;

impl ReceiverBindingOrigins {
    // Borrow an own-data certificate only; never expose mutable value provenance.
    pub(in super::super) fn own_data_member(
        &self,
        scope: usize,
        name: &str,
        read: nested::RootRead,
        member: &str,
    ) -> Option<u32> {
        let root = self.roots.get(&scope)?.get(name)?;
        if root.owners.len() != 1 {
            return None;
        }
        let (completed_at, selected) = root
            .owners
            .get(&read.1)?
            .snapshots
            .range(..=read.0)
            .next_back()?;
        let selected = root
            .patterns
            .as_ref()
            .and_then(|patterns| patterns.selected(read))
            .unwrap_or(Some(*selected))?;
        matches!(&self.arena[selected], Origin::Value(ValueExpression::Object(fields, _))
            if fields.contains_key(member))
        .then_some(*completed_at)
    }
}
