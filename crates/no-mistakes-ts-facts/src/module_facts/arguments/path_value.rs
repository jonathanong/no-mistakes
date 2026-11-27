#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PathValueKind {
    Unknown,
    Scalar,
    Mutable,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemberEffect {
    pub path: Vec<String>,
    pub escaped: bool,
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
    pub fn requirement(&self, requested: &[&str]) -> Option<bool> {
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
