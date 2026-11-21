use super::StringOrList;

impl StringOrList {
    pub fn values(&self) -> Vec<String> {
        match self {
            Self::One(s) => vec![s.clone()],
            Self::Many(v) => v.clone(),
        }
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
