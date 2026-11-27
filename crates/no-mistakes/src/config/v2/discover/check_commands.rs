use super::*;
use crate::config::source_locations::ConfigPath;

pub(super) fn validate(config: &NoMistakesConfig) -> Result<()> {
    for (index, command) in config.checks.commands.iter().enumerate() {
        for (field, patterns) in [("include", &command.include), ("exclude", &command.exclude)] {
            path_globs::validate_globs(
                patterns,
                &format!("checks.commands[{index}].{field}"),
                |pattern| {
                    ConfigPath::default()
                        .key("checks")
                        .key("commands")
                        .index(index)
                        .key(field)
                        .index(pattern)
                },
            )?;
        }
        if command
            .command
            .first()
            .is_none_or(|executable| executable.trim().is_empty())
        {
            anyhow::bail!(
                "checks.commands[{index}].command must start with a non-blank executable token"
            );
        }
        if command.always && command.file_args != super::super::schema::CheckFileArgs::None {
            anyhow::bail!(
                "checks.commands[{index}].always runs once for the whole project, so file selection is invalid; set fileArgs: none"
            );
        }
        if command.always && (!command.include.is_empty() || !command.exclude.is_empty()) {
            anyhow::bail!(
                "checks.commands[{index}].always runs once for the whole project, so file selection is invalid; remove include and exclude"
            );
        }
    }
    Ok(())
}
