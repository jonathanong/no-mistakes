use crate::codebase::structured_value::merge_keys::{expand_located, KeyLocation, MergeError};
use crate::codebase::structured_value::{PartialYaml, StructuredValueError};
use crate::fx::FxHashMap;
use serde_yaml::Value;

pub(super) struct LocatedKeys {
    line: usize,
    keys: FxHashMap<*const u8, KeyLocation>,
    pub(super) incomplete: bool,
}

impl LocatedKeys {
    pub(super) fn new(line: usize, incomplete: bool) -> Self {
        Self {
            line,
            keys: FxHashMap::default(),
            incomplete,
        }
    }

    pub(super) fn record(&mut self, key: &str, location: Option<KeyLocation>) {
        match location {
            Some(location) => {
                self.keys.insert(key.as_ptr(), location);
            }
            None => self.incomplete = true,
        }
    }

    pub(super) fn expand(&self, value: &mut Value) -> Result<(), MergeError> {
        if self.incomplete {
            return Err(MergeError::MissingLocation(self.line));
        }
        expand_located(value, &|key| {
            self.lookup(key)
                .ok_or(MergeError::MissingLocation(self.line))
        })
        .map(|_| ())
    }

    pub(super) fn expand_scoped(&self, mut value: Value) -> Result<Value, StructuredValueError> {
        if !self.incomplete {
            self.expand(&mut value)
                .map_err(StructuredValueError::merge)?;
            return Ok(value);
        }

        let mut value = Box::new(value);
        let unresolved = expand_located(&mut value, &|key| {
            Ok(self.lookup(key).unwrap_or(KeyLocation::Unlocated))
        })
        .map_err(StructuredValueError::merge)?;
        Err(StructuredValueError::unlocated(
            self.line,
            PartialYaml::new(value, unresolved),
        ))
    }

    // Expansion moves the decoded strings while retaining their heap allocations.
    fn lookup(&self, key: &str) -> Option<KeyLocation> {
        self.keys.get(&key.as_ptr()).copied()
    }
}
