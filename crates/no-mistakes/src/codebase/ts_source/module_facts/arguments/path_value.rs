#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum PathValueKind {
    Unknown,
    Scalar,
    Mutable,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct MemberEffect {
    pub(crate) path: Vec<String>,
    pub(crate) escaped: bool,
}

impl MemberEffect {
    pub(super) fn unknown() -> Self {
        Self {
            path: Vec::new(),
            escaped: false,
        }
    }
}

impl MemberEffect {
    pub(crate) fn requirement(&self, requested: &[&str]) -> Option<bool> {
        let shared = self
            .path
            .iter()
            .zip(requested)
            .all(|(left, right)| left == right);
        if shared && (!self.escaped || self.path.len() < requested.len()) {
            return None;
        }
        Some(shared && self.escaped && self.path.len() == requested.len())
    }
}
