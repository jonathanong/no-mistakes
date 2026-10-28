//! Exact local call census for one finite helper-name edge, memoized per owner.
use super::{index::Index, Argument, Root};
use crate::codebase::postgres::query_annotation::heads::{bindings::Bindings, syntax::Syntax};
use crate::codebase::ts_source::module_facts::ModuleBinding;
impl Index<'_> {
    pub(super) fn helper_domain(
        &mut self,
        syntax: &Syntax,
        bindings: &Bindings<'_>,
        root: &Root,
    ) -> Option<&[String]> {
        let function = *self.functions.get(&root.function)?;
        let helper = self.declared(function.declaration)?;
        let parameter = self.declared(function.parameter)?;
        if bindings.owner(root.reference)?.id != parameter.id
            || syntax.append_tags.get(&root.declaration) != Some(&root.tag)
            || !bindings.trusted_root(root.declaration)
        {
            return None;
        }
        if !self.domains.contains_key(&helper.id) {
            self.work.domains += 1;
            let domain = self.compute_domain(syntax, bindings, helper, parameter);
            self.domains.insert(helper.id, domain);
        }
        self.domains.get(&helper.id)?.as_deref()
    }
    fn compute_domain(
        &mut self,
        syntax: &Syntax,
        bindings: &Bindings<'_>,
        helper: &ModuleBinding,
        parameter: &ModuleBinding,
    ) -> Option<Vec<String>> {
        if !self.stable(syntax, bindings, helper) || !self.stable(syntax, bindings, parameter) {
            return None;
        }
        let calls = self.calls.get(&helper.id)?;
        let [call] = calls.as_slice() else {
            return None;
        };
        let call = *call;
        if !self.only_references(helper, &[call.callee]) || !call.valid || call.count == 0 {
            return None;
        }
        let Argument::Reference(argument) = &call.first else {
            return None;
        };
        let select = *self.selects.get(&call.context?)?;
        let select_parameter = self.declared(select.parameter)?;
        if bindings
            .owner(*argument)
            .is_none_or(|owner| owner.id != select_parameter.id)
            || !self.stable(syntax, bindings, select_parameter)
        {
            return None;
        }
        let select_binding = self.declared(select.declaration)?;
        if !self.stable(syntax, bindings, select_binding) {
            return None;
        }
        let calls = self.calls.get(&select_binding.id)?;
        if calls.is_empty() || calls.len() > 16 {
            return None;
        }
        // At most 16 borrowed call pointers; never clone an unbounded call table.
        let calls = calls.to_vec();
        if !self.only_references(
            select_binding,
            &calls.iter().map(|call| call.callee).collect::<Vec<_>>(),
        ) {
            return None;
        }
        let mut values = Vec::new();
        for call in calls {
            if !call.valid || call.count != 1 || call.context != Some(select.enclosing) {
                return None;
            }
            let Argument::Literal(value) = &call.first else {
                return None;
            };
            if value.trim().is_empty() {
                return None;
            }
            if !values.contains(value) {
                values.push(value.clone());
            }
        }
        Some(values)
    }
}
