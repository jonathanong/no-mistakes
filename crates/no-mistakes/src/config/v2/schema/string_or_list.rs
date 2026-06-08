use super::StringOrList;

impl StringOrList {
    pub fn values(&self) -> Vec<String> {
        match self {
            Self::One(s) => vec![s.clone()],
            Self::Many(v) => v.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_values_one() {
        let single = StringOrList::One("hello".to_string());
        assert_eq!(single.values(), vec!["hello".to_string()]);
    }

    #[test]
    fn test_values_many() {
        let multiple = StringOrList::Many(vec!["hello".to_string(), "world".to_string()]);
        assert_eq!(
            multiple.values(),
            vec!["hello".to_string(), "world".to_string()]
        );
    }

    #[test]
    fn test_values_many_empty() {
        let empty = StringOrList::Many(vec![]);
        let expected: Vec<String> = vec![];
        assert_eq!(empty.values(), expected);
    }
}
