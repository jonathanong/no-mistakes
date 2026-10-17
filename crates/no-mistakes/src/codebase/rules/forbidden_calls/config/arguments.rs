use anyhow::{bail, Result};
use serde::Deserialize;

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(in super::super) enum UnknownArguments {
    #[default]
    Finding,
    Ignore,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in super::super) struct ArgumentConditions {
    pub(in super::super) all: Vec<ArgumentPredicate>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(untagged)]
pub(in super::super) enum Literal {
    String(String),
    Boolean(bool),
    Number(f64),
    Null(()),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(in super::super) enum ArgumentPredicate {
    Equals(Equals),
    Includes(Includes),
    Excludes(Excludes),
}

macro_rules! predicate {
    ($name:ident, $field:ident, $key:literal) => {
        #[derive(Debug, Deserialize)]
        #[serde(deny_unknown_fields)]
        pub(in super::super) struct $name {
            pub(in super::super) position: usize,
            pub(in super::super) property: Option<String>,
            #[serde(rename = $key)]
            pub(in super::super) $field: Literal,
        }
    };
}
predicate!(Equals, equals, "equals");
predicate!(Includes, includes, "arrayIncludes");
predicate!(Excludes, excludes, "arrayExcludes");

impl ArgumentPredicate {
    pub(in super::super) fn selector(&self) -> (usize, Option<&str>) {
        match self {
            Self::Equals(p) => (p.position, p.property.as_deref()),
            Self::Includes(p) => (p.position, p.property.as_deref()),
            Self::Excludes(p) => (p.position, p.property.as_deref()),
        }
    }
}

pub(in super::super) fn validate(conditions: &ArgumentConditions) -> Result<()> {
    if conditions.all.is_empty() || conditions.all.len() > 16 {
        bail!("forbidden-calls: whenArguments.all requires between 1 and 16 predicates");
    }
    for predicate in &conditions.all {
        let literal = match predicate {
            ArgumentPredicate::Equals(p) => &p.equals,
            ArgumentPredicate::Includes(p) => &p.includes,
            ArgumentPredicate::Excludes(p) => &p.excludes,
        };
        if matches!(literal, Literal::Number(number) if !number.is_finite())
            || matches!(literal, Literal::String(value) if value.len() > 4096)
        {
            bail!("forbidden-calls: argument literals require finite numbers and strings of at most 4096 bytes");
        }
        let (position, property) = predicate.selector();
        if position >= 32 {
            bail!("forbidden-calls: argument position must be between 0 and 31");
        }
        if property.is_some_and(|name| name.is_empty() || name.len() > 128) {
            bail!("forbidden-calls: argument property requires a nonempty literal name of at most 128 bytes");
        }
    }
    Ok(())
}
