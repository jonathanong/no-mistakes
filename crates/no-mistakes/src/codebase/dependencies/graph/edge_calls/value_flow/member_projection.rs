/// Borrowed use-site state: never stored in parameter agreements or keys.
/// Nodes follow evaluation inward; histories expect outermost member first.
pub(super) struct MemberProjection<'a> {
    pub(super) member: &'a str,
    pub(super) outer: Option<&'a MemberProjection<'a>>,
    pub(super) transparent_alias: bool,
    // Shared only by this member evaluation; barriers can only turn it off.
    pub(super) whole_receiver: &'a std::cell::Cell<bool>,
}
impl MemberProjection<'_> {
    pub(super) fn binding_hint(
        hint: Option<&Self>,
        binding: Option<&super::ValueBinding>,
    ) -> Option<Self> {
        let hint = hint?;
        let binding = binding?;
        let transparent_alias = matches!(&binding.value,
            super::ValueExpression::Named { name, .. } if !name.contains('.'));
        if !transparent_alias {
            hint.whole_receiver.set(false);
        }
        // Reset at copied/nontransparent initializers, even through an outer
        // identifier alias. Parameter computation receives no hint.
        Some(Self {
            member: hint.member,
            outer: hint.outer,
            transparent_alias,
            whole_receiver: hint.whole_receiver,
        })
    }
    pub(super) fn source_alias_seed(hint: Option<&Self>, length: usize) -> usize {
        if hint.is_some_and(|hint| hint.transparent_alias) {
            length
        } else {
            0
        }
    }
    pub(super) fn members(&self) -> Vec<&str> {
        let mut members = Vec::new();
        let mut hint = Some(self);
        while let Some(current) = hint {
            members.push(current.member);
            hint = current.outer;
        }
        members.reverse();
        members
    }
}

pub(super) fn preserve_whole_receiver(
    hint: Option<&MemberProjection<'_>>,
    value: &super::FlowValue,
    origins: Option<&super::OriginNode>,
) {
    if let Some(hint) = hint {
        if !matches!(
            value.executor(),
            super::FlowValue::Target(super::ResolvedCallTarget::ModuleExport { .. })
        ) || origins.is_some_and(|origin| origin.has_receiver_snapshots)
        {
            hint.whole_receiver.set(false);
        }
    }
}
