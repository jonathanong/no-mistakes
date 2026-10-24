use crate::config::source_locations::ConfigPath;

/// Keep the public diagnostic spelling separate from the authored YAML shape.
#[derive(Clone)]
pub(super) struct Field {
    pub(super) label: String,
    pub(super) origin: Option<ConfigPath>,
    pub(super) rule_index: Option<usize>,
}

impl Field {
    pub(super) fn new(key: &str, origins: bool) -> Self {
        Self {
            label: key.to_string(),
            origin: origins.then(|| ConfigPath::default().key(key)),
            rule_index: None,
        }
    }

    pub(super) fn key(&self, key: &str) -> Self {
        Self {
            label: format!("{}.{key}", self.label),
            origin: self.origin.as_ref().map(|path| path.clone().key(key)),
            rule_index: self.rule_index,
        }
    }

    pub(super) fn index(&self, index: usize) -> Self {
        self.index_at(index, index)
    }

    pub(super) fn index_at(&self, displayed: usize, authored: usize) -> Self {
        Self {
            label: format!("{}[{displayed}]", self.label),
            origin: self
                .origin
                .as_ref()
                .map(|path| path.clone().index(authored)),
            rule_index: if self.label == "rules" {
                Some(displayed)
            } else {
                self.rule_index
            },
        }
    }

    pub(super) fn authored_key(mut self, key: &str) -> Self {
        self.origin = self.origin.map(|path| path.key(key));
        self
    }
}
