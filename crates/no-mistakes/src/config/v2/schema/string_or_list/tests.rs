use super::super::StringOrList;

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
