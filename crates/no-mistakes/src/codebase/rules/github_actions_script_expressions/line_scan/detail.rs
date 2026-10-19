use super::super::expressions::expressions;
use super::super::hit::Hit;
use super::super::yaml_parse::BlockRole;
use super::super::yaml_text::{ends_inside_quote, is_github_script};
use super::Scanner;

pub(crate) struct StepUse {
    pub(crate) job: String,
    pub(crate) step: usize,
    pub(crate) line: usize,
}

pub(crate) fn step_uses_lines(source: &str) -> Vec<StepUse> {
    let lines: Vec<&str> = source.lines().collect();
    let mut scanner = Scanner::from_lines(&lines, false, false, true);
    scanner.scan();
    scanner.located
}

pub(super) fn begin_job(scanner: &mut Scanner<'_>, name: &str) {
    if !scanner.locate {
        return;
    }
    scanner.job = name.to_string();
    scanner.step = 0;
}

pub(super) fn begin_step(scanner: &mut Scanner<'_>) {
    if !scanner.locate {
        return;
    }
    scanner.step += 1;
    scanner.uses_line = None;
}

pub(super) fn note_uses(scanner: &mut Scanner<'_>, line: usize) {
    if scanner.locate {
        scanner.uses_line = Some(line);
    }
}

pub(super) fn end_step(scanner: &mut Scanner<'_>) {
    if !scanner.locate {
        return;
    }
    if let Some(line) = scanner.uses_line.take() {
        scanner.located.push(StepUse {
            job: scanner.job.clone(),
            step: scanner.step,
            line,
        });
    }
}

pub(super) fn record_inline(
    scanner: &mut Scanner<'_>,
    role: BlockRole,
    line_no: usize,
    value: &str,
) {
    push_line(scanner, role, line_no, value);
    let trimmed = value.trim();
    let bytes = trimmed.as_bytes();
    // Callers only reach here with a non-empty scalar, so the first byte exists.
    if bytes
        .first()
        .is_some_and(|byte| *byte == b'"' || *byte == b'\'')
    {
        let quote = bytes[0] as char;
        if ends_inside_quote(trimmed, false, quote) {
            scanner.open_quote = Some((role, quote));
        }
    }
}

pub(super) fn push_line(scanner: &mut Scanner<'_>, role: BlockRole, line_no: usize, text: &str) {
    let slot = match role {
        BlockRole::Run => &mut scanner.run_lines,
        BlockRole::Script => &mut scanner.script_lines,
        BlockRole::Other => return,
    };
    slot.push((line_no, text.to_string()));
}

pub(super) fn ends_quote(line: &str, quote: char) -> bool {
    ends_inside_quote(line, true, quote)
}

pub(super) fn flush_step(scanner: &mut Scanner<'_>) {
    let uses = scanner.uses.take();
    let run_lines = std::mem::take(&mut scanner.run_lines);
    let script_lines = std::mem::take(&mut scanner.script_lines);
    for (line, text) in run_lines {
        for expr in expressions(&text) {
            scanner.hits.push(Hit {
                line,
                expr,
                script: false,
            });
        }
    }
    if uses.as_deref().is_some_and(is_github_script) {
        for (line, text) in script_lines {
            for expr in expressions(&text) {
                scanner.hits.push(Hit {
                    line,
                    expr,
                    script: true,
                });
            }
        }
    }
}
