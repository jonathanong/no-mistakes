use super::*;

impl ParsedRunnerConfigs {
    pub(crate) fn file_facts(&self) -> &BTreeMap<PathBuf, RunnerConfigFileFacts> {
        &self.files
    }
}
