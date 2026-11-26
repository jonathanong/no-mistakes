use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct ReactTraitsConfig {
    pub frontend_root: Option<String>,
    pub assert_no_fetch: Option<bool>,
}
