use super::super::RULE_ID;
use super::FunctionSelector;
use crate::config::v2::schema::StringOrList;
use anyhow::{bail, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum Root {
    File(FileRoot),
    Module(ModuleRoot),
    Function(FunctionRoot),
    Vitest(CatalogRoot),
    Playwright(PlaywrightRoot),
    Glob(GlobRoot),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FileRoot {
    pub(crate) file: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ModuleRoot {
    pub(crate) module: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FunctionRoot {
    pub(crate) function: FunctionSelector,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CatalogRoot {
    pub(crate) vitest: CatalogSelector,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PlaywrightRoot {
    pub(crate) playwright: CatalogSelector,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GlobRoot {
    pub(crate) glob: StringOrList,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum CatalogSelector {
    All(bool),
    Projects(Vec<String>),
}

pub(crate) fn catalog_names(selector: &CatalogSelector) -> Vec<String> {
    match selector {
        CatalogSelector::All(true) => Vec::new(),
        CatalogSelector::All(false) => unreachable!("validated options"),
        CatalogSelector::Projects(names) => names.clone(),
    }
}

pub(super) fn validate_roots(roots: &[Root]) -> Result<()> {
    for root in roots {
        match root {
            Root::Vitest(CatalogRoot {
                vitest: CatalogSelector::All(false),
            }) => bail!("{RULE_ID}: vitest root must be true or a non-empty project list"),
            Root::Vitest(CatalogRoot {
                vitest: CatalogSelector::Projects(names),
            }) if names.is_empty() => {
                bail!("{RULE_ID}: vitest root project list must not be empty")
            }
            Root::Playwright(PlaywrightRoot {
                playwright: CatalogSelector::All(false),
            }) => bail!("{RULE_ID}: playwright root must be true or a non-empty project list"),
            Root::Playwright(PlaywrightRoot {
                playwright: CatalogSelector::Projects(names),
            }) if names.is_empty() => {
                bail!("{RULE_ID}: playwright root project list must not be empty")
            }
            Root::Glob(GlobRoot { glob }) => validate_glob(glob)?,
            _ => {}
        }
    }
    Ok(())
}

fn validate_glob(glob: &StringOrList) -> Result<()> {
    let patterns = glob.values();
    if patterns.is_empty() || patterns.iter().any(String::is_empty) {
        bail!("{RULE_ID}: glob root must be a non-empty pattern or list");
    }
    Ok(())
}
