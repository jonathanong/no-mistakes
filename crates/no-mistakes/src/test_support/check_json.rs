use std::path::PathBuf;

use anyhow::Result;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct TestCheckOptions {
    effort: Option<crate::config::v2::Effort>,
    root: Option<String>,
    tsconfig: Option<String>,
    config: Option<String>,
    include_suppressed: bool,
    include_runner_config_deadlines: bool,
    include_sql_verdicts: bool,
}

pub trait CheckJsonArg {
    fn into_json_value(self) -> Value;
}

impl CheckJsonArg for Value {
    fn into_json_value(self) -> Value {
        self
    }
}

impl CheckJsonArg for String {
    fn into_json_value(self) -> Value {
        serde_json::from_str(&self).expect("valid test check options JSON")
    }
}

pub fn check_json(options: impl CheckJsonArg) -> Result<String> {
    let options = serde_json::from_value::<TestCheckOptions>(options.into_json_value())?;
    let root = PathBuf::from(options.root.expect("test check_json requires root"));
    let results = crate::check_runner::run_all_with_options(
        root,
        options.config.map(PathBuf::from),
        options.tsconfig.map(PathBuf::from),
        options.include_suppressed,
        options.include_runner_config_deadlines,
        crate::check_runner::AnalysisOptions {
            effort: options.effort,
            include_sql_verdicts: options.include_sql_verdicts,
        },
    )?;
    Ok(crate::cli::json_string(&crate::check_runner::json_value(
        &results,
    )))
}
