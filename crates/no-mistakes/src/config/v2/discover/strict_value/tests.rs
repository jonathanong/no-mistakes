use super::StrictValue;
use serde::de::DeserializeOwned;
use serde::Deserialize;

const FIXTURE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-cases/config-v2/strict-root-project/fixture/typed-collections.yml"
));
const NULL_FIXTURE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test-cases/config-v2/strict-root-project/fixture/4477-null.yml"
));

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct TypedCollections {
    tuple: (String, bool),
    tuple_struct: TaggedPair,
    wrapped_list: WrappedList,
    unit: UnitSetting,
}

#[derive(Debug, Deserialize, PartialEq)]
struct TaggedPair(String, usize);

#[derive(Debug, Deserialize, PartialEq)]
struct WrappedList(Vec<String>);

#[derive(Debug, Deserialize, PartialEq)]
struct UnitSetting;

fn decode<T: DeserializeOwned>(source: &str) -> Result<T, serde_yaml::Error> {
    T::deserialize(StrictValue(serde_yaml::from_str(source)?))
}

#[test]
fn retains_tuple_and_newtype_shapes_with_yaml_tags() {
    let value: TypedCollections = decode(FIXTURE).unwrap();
    assert_eq!(
        value,
        TypedCollections {
            tuple: ("example".to_string(), true),
            tuple_struct: TaggedPair("fixture".to_string(), 2),
            wrapped_list: WrappedList(vec!["src/example.ts".to_string()]),
            unit: UnitSetting,
        }
    );
}

#[test]
fn rejects_null_even_when_a_collection_has_a_newtype_wrapper() {
    let error = decode::<WrappedList>(NULL_FIXTURE).unwrap_err();
    assert_eq!(error.to_string(), "invalid type: null, expected a sequence");
}
