use super::super::yaml_parse::BlockRole;
use super::super::yaml_text::is_github_script;
use super::Scanner;

pub(crate) struct StepScript {
    pub(crate) line: usize,
    pub(crate) count: usize,
    pub(crate) github_script: bool,
}

#[derive(Default)]
pub(super) struct ScriptCollector {
    enabled: bool,
    run: Option<usize>,
    script: Option<usize>,
    collected: Vec<StepScript>,
}

impl ScriptCollector {
    pub(super) fn note_header(&mut self, role: Option<BlockRole>, line: usize) {
        if !self.enabled {
            return;
        }
        match role {
            Some(BlockRole::Run) => self.run = Some(line),
            Some(BlockRole::Script) => self.script = Some(line),
            _ => {}
        }
    }

    pub(super) fn flush(
        &mut self,
        uses: Option<&str>,
        runs: &[(usize, String)],
        scripts: &[(usize, String)],
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
            if let Some(line) = header {
                self.collected.push(StepScript {
                    line,
                    count: body
                        .iter()
                        .filter(|(_, text)| !text.trim().is_empty())
                        .count(),
                    github_script,
                });
            }
        }
    }
}

pub(crate) fn step_script_lines(source: &str, action: bool) -> Vec<StepScript> {
    let lines: Vec<_> = source.lines().collect();
    let mut scanner = Scanner::from_lines(&lines, action, action, false);
    scanner.scripts.enabled = true;
    scanner.scan();
    scanner.scripts.collected
}
