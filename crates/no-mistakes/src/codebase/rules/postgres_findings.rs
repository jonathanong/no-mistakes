use super::RuleFinding;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct LocatedRuleFinding {
    pub(crate) finding: RuleFinding,
    pub(crate) source_offset: Option<usize>,
}

impl From<RuleFinding> for LocatedRuleFinding {
    fn from(finding: RuleFinding) -> Self {
        Self {
            finding,
            source_offset: None,
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct PostgresFindings(pub(crate) Vec<LocatedRuleFinding>);

impl PostgresFindings {
    pub(crate) fn push(&mut self, finding: RuleFinding) {
        self.0.push(finding.into());
    }

    pub(crate) fn push_at(&mut self, finding: RuleFinding, source_offset: Option<usize>) {
        self.0.push(LocatedRuleFinding {
            finding,
            source_offset,
        });
    }

    pub(crate) fn extend<T: Into<LocatedRuleFinding>>(
        &mut self,
        findings: impl IntoIterator<Item = T>,
    ) {
        self.0.extend(findings.into_iter().map(Into::into));
    }

    pub(crate) fn get_mut(&mut self, index: usize) -> &mut RuleFinding {
        &mut self.0[index].finding
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn sort(&mut self) {
        self.0.sort();
        self.0.dedup();
    }

    pub(crate) fn finish(mut self) -> Vec<RuleFinding> {
        self.sort();
        self.0.into_iter().map(|located| located.finding).collect()
    }
}

impl IntoIterator for PostgresFindings {
    type Item = LocatedRuleFinding;
    type IntoIter = std::vec::IntoIter<LocatedRuleFinding>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<LocatedRuleFinding> for PostgresFindings {
    fn from_iter<T: IntoIterator<Item = LocatedRuleFinding>>(findings: T) -> Self {
        Self(findings.into_iter().collect())
    }
}
