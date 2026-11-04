use super::super::yaml_parse::BlockRole;
use super::super::yaml_text::is_github_script;
use super::Scanner;

pub(crate) struct StepScript {
    pub(crate) line: usize,
    pub(crate) count: usize,
    pub(crate) github_script: bool,
    pub(crate) supported: bool,
}

#[derive(Default)]
pub(super) struct ScriptCollector {
    enabled: bool,
    run: Option<(usize, bool)>,
    script: Option<(usize, bool)>,
    collected: Vec<StepScript>,
}

impl ScriptCollector {
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
                self.collected.push(StepScript {
                    line,
                    count: count_lines(body, source_lines),
                    github_script,
                    supported,
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

fn count_lines(body: &[(usize, String)], source_lines: &[&str]) -> usize {
    let mut count = 0;
    let mut continued = false;
    for (line, text) in body {
        if !text.trim().is_empty() && !continued {
            count += 1;
        }
        continued = source_lines
            .get(line - 1)
            .is_some_and(|text| text.ends_with('\\'));
    }
    count
}
