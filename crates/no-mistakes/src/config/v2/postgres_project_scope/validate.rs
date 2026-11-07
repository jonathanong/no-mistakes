use super::narrow::validate_glob;
use crate::config::v2::schema::NoMistakesConfig;
use anyhow::{bail, Result};

pub(super) fn project_options(config: &NoMistakesConfig) -> Result<()> {
    for (name, project) in &config.projects {
        if let Some(postgres) = &project.postgres {
            if postgres.executors.as_ref().is_some_and(|executors| {
                executors.executor_names.is_some() && executors.extra_executor_names.is_some()
            }) {
                bail!("projects.{name}.postgres.executors executorNames and extraExecutorNames cannot both be set; use extraExecutorNames to extend the defaults");
            }
            for pattern in &postgres.sql_include {
                validate_glob(pattern, &format!("projects.{name}.postgres.sqlInclude"))?;
            }
        }
    }
    Ok(())
}
