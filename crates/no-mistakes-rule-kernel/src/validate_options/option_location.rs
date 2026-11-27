#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Segment {
    Key(String),
    Index(usize),
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct ConfigPath(Vec<Segment>);

impl ConfigPath {
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.0.push(Segment::Key(key.into()));
        self
    }

    pub fn index(mut self, index: usize) -> Self {
        self.0.push(Segment::Index(index));
        self
    }

    pub fn pop(&mut self) -> Option<Segment> {
        self.0.pop()
    }

    pub fn as_slice(&self) -> &[Segment] {
        &self.0
    }
}

#[derive(Debug)]
pub struct OptionLocation {
    pub path: ConfigPath,
    message: String,
}

impl std::fmt::Display for OptionLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for OptionLocation {}

impl OptionLocation {
    pub fn semantic(path: ConfigPath, message: String) -> anyhow::Error {
        Self { path, message }.into()
    }
}
