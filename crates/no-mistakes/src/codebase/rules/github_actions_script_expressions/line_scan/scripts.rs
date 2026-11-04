use super::super::yaml_parse::BlockRole;
use super::super::yaml_text::is_github_script;
use super::Scanner;
use crate::fx::FxHashMap;

mod scalar;

pub(crate) struct StepScript {
    pub(crate) job: String,
    pub(crate) step: usize,
    pub(crate) line: usize,
    pub(crate) count: usize,
    pub(crate) github_script: bool,
    pub(crate) supported: bool,
}

#[derive(Default)]
pub(crate) struct ScriptScan {
    pub(crate) scripts: Vec<StepScript>,
    pub(crate) step_lines: FxHashMap<(String, usize), usize>,
}

#[derive(Default)]
pub(super) struct ScriptCollector {
    enabled: bool,
    run: Option<(usize, bool)>,
    script: Option<(usize, bool)>,
    collected: ScriptScan,
}

impl ScriptCollector {
    pub(super) fn note_step(&mut self, job: &str, step: usize, line: usize) {
        if self.enabled {
            self.collected
                .step_lines
                .insert((job.to_string(), step), line);
        }
    }

    pub(super) fn collects_plain(&self, value: &str) -> bool {
        self.enabled && !value.starts_with(['\'', '"'])
    }

    pub(super) fn note_header(&mut self, role: Option<BlockRole>, line: usize, value: &str) {
        if !self.enabled {
            return;
        }
        match role {
            Some(BlockRole::Run) => self.run = Some((line, !value.starts_with(['*', '&']))),
            Some(BlockRole::Script) => self.script = Some((line, !value.starts_with(['*', '&']))),
            _ => {}
        }
    }

    pub(super) fn flush(
        &mut self,
        job: &str,
        step: usize,
        uses: Option<&str>,
        runs: &[(usize, String)],
        scripts: &[(usize, String)],
        source_lines: &[&str],
    ) {
        if !self.enabled {
            return;
        }
        for (header, body, github_script) in [
            (self.run.take(), runs, false),
            (self.script.take(), scripts, true),
        ] {
            if github_script && !uses.is_some_and(is_github_script) {
                continue;
            }
            if let Some((line, supported)) = header {
                self.collected.scripts.push(StepScript {
                    job: job.to_string(),
                    step,
                    line,
                    count: scalar::count_lines(line, body, source_lines),
                    github_script,
                    supported,
                });
            }
        }
    }
}

pub(crate) fn step_script_lines(source: &str, action: bool) -> ScriptScan {
    let lines: Vec<_> = source.lines().collect();
    let mut scanner = Scanner::from_lines(&lines, action, action, true);
    scanner.scripts.enabled = true;
    scanner.scan();
    scanner.scripts.collected
}
