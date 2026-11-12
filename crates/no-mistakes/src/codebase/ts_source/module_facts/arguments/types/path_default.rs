use super::PathOrigin;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct PathDefault {
    pub(crate) value: Box<PathOrigin>,
    pub(crate) nullish: bool,
}
