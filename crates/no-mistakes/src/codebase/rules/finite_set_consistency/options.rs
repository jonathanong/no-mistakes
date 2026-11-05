use super::{Options, RULE_ID};
use anyhow::{ensure, Result};

impl Options {
    pub(super) fn validate(&self) -> Result<()> {
        for (index, spec) in self.sets.iter().enumerate() {
            ensure!(
                !spec.name.trim().is_empty(),
                "{RULE_ID}: sets[{index}].name is required and must not be empty"
            );
            ensure!(
                !spec.kind.trim().is_empty(),
                "{RULE_ID}: sets[{index}].kind is required and must not be empty"
            );
        }
        for (index, comparison) in self.comparisons.iter().enumerate() {
            ensure!(
                !comparison.left.trim().is_empty(),
                "{RULE_ID}: comparisons[{index}].left is required and must not be empty"
            );
            ensure!(
                !comparison.right.trim().is_empty(),
                "{RULE_ID}: comparisons[{index}].right is required and must not be empty"
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
