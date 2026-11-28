use super::Options;
use crate::codebase::rules::{path_filter::GlobMatcher, RuleFinding};
use anyhow::Result;

pub(super) fn finding(
    id: &str,
    file: &str,
    line: usize,
    target: &str,
    message: String,
) -> RuleFinding {
    RuleFinding {
        causes: Vec::new(),
        rule: id.into(),
        file: file.into(),
        line,
        message,
        import: None,
        target: Some(target.into()),
    }
}

pub(super) fn message(bash: bool, target: &str) -> String {
    if bash {
        format!(
            "Array `{target}` can be empty. Bash before 4.4 treats an empty array as unbound under nounset. Wrap the quoted expansion in ${{{target}[@]+...}} to guard it and keep the original array selector."
        )
    } else {
        "Zsh reserves `status` as the read-only exit status. Rename the variable to `exit_status`."
            .into()
    }
}

pub(in crate::codebase::rules) fn validate_options(id: &str, options: &Options) -> Result<()> {
    GlobMatcher::new(&options.files, &format!("{id} options.files"))?;
    Ok(())
}
