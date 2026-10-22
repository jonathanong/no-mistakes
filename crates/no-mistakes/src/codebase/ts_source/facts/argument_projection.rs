use std::sync::Arc;

/// Coverage of literal argument facts retained by one request's shared parse.
/// Exact names are sorted at construction, including for Rust API callers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ArgumentProjection(Projection);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
enum Projection {
    #[default]
    Full,
    Exact(Arc<[String]>),
}

impl ArgumentProjection {
    pub fn exact(mut names: Vec<String>) -> Self {
        names.sort();
        names.dedup();
        Self(Projection::Exact(names.into()))
    }

    pub fn covers(&self, required: &Self) -> bool {
        match (&self.0, &required.0) {
            (Projection::Full, _) => true,
            (Projection::Exact(have), Projection::Exact(need)) => {
                need.iter().all(|name| have.binary_search(name).is_ok())
            }
            (Projection::Exact(_), Projection::Full) => false,
        }
    }

    pub fn include(&mut self, other: &Self) {
        match (&self.0, &other.0) {
            (Projection::Full, _) | (_, Projection::Full) => self.0 = Projection::Full,
            (Projection::Exact(have), Projection::Exact(need)) => {
                let mut names = have.to_vec();
                names.extend(need.iter().cloned());
                *self = Self::exact(names);
            }
        }
    }

    pub(crate) fn is_exact(&self) -> bool {
        matches!(self.0, Projection::Exact(_))
    }

    pub(crate) fn matches(&self, callee: &str) -> bool {
        match &self.0 {
            Projection::Full => true,
            Projection::Exact(names) => names
                .binary_search_by(|name| name.as_str().cmp(callee))
                .is_ok(),
        }
    }
}

#[cfg(test)]
mod tests;
