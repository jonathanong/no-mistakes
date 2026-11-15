#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ConditionalPlace {
    pub(crate) hook: Option<(u32, u32)>,
    pub(crate) function: Option<(u32, u32)>,
    pub(crate) program: bool,
    pub(crate) branch: bool,
}

impl ConditionalPlace {
    pub(crate) const fn unmarked() -> Self {
        Self {
            hook: None,
            function: None,
            program: false,
            branch: false,
        }
    }
}
