/// Borrowed use-site state: never stored in parameter agreements or keys.
/// Nodes follow evaluation inward; histories expect outermost member first.
pub(super) struct MemberProjection<'a> {
    pub(super) member: &'a str,
    pub(super) outer: Option<&'a MemberProjection<'a>>,
}
impl MemberProjection<'_> {
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
