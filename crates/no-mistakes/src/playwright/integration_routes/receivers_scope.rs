/// Registration identity is lexical (AST span), never a potentially duplicate title.
#[derive(Clone, Default, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct RegistrationScope {
    pub(super) describes: Vec<u32>,
    pub(super) test: Option<u32>,
    pub(super) hook: Option<u32>,
}

impl RegistrationScope {
    fn applies_to(&self, consuming: &Self) -> bool {
        consuming.describes.starts_with(&self.describes)
            && self.test.is_none_or(|test| {
                self.describes == consuming.describes && consuming.test == Some(test)
            })
    }
}
