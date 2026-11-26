use super::PathOrigin;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathDefault {
    pub value: Box<PathOrigin>,
    pub nullish: bool,
}
