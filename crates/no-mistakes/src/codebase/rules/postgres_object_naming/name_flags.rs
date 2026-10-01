#[derive(Clone, Copy, Default)]
pub(super) struct NameFlags {
    pub(super) tokens: bool,
    pub(super) plural: Option<&'static str>,
    pub(super) underscore: bool,
}

impl NameFlags {
    pub(super) fn tokens() -> Self {
        Self {
            tokens: true,
            plural: None,
            underscore: false,
        }
    }

    pub(super) fn table() -> Self {
        Self {
            tokens: true,
            plural: Some("table"),
            underscore: true,
        }
    }

    pub(super) fn underscore() -> Self {
        Self {
            tokens: true,
            plural: None,
            underscore: true,
        }
    }

    pub(super) fn enum_name() -> Self {
        Self {
            tokens: true,
            plural: Some("enum"),
            underscore: true,
        }
    }
}
