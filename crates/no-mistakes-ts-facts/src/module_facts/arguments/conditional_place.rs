#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConditionalPlace {
    pub hook: Option<(u32, u32)>,
    pub function: Option<(u32, u32)>,
    pub program: bool,
    pub branch: bool,
}

impl ConditionalPlace {
    pub const fn unmarked() -> Self {
        Self {
            hook: None,
            function: None,
            program: false,
            branch: false,
        }
    }
}
