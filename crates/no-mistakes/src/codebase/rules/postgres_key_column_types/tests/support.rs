use super::super::compile::compile;
use super::super::{Options, RULE_ID};

pub(super) const PATH: &str = "schemaCatalogPath: schema.json\n";

pub(super) fn expect_err(yaml: &str, snippet: &str) {
    let options: Options = serde_yaml::from_str(yaml).unwrap();
    let error = match compile(&options, None) {
        Err(error) => error.to_string(),
        Ok(_) => panic!("expected config error containing {snippet}"),
    };
    assert!(error.contains(&format!("{RULE_ID} option")), "{error}");
    assert!(error.contains(snippet), "{error}");
}
