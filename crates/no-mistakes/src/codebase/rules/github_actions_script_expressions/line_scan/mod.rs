use super::hit::Hit;
use super::yaml_parse::{
    classify_key, next_is_child_key, parse_line, script_role, BlockRole, Frame, Kind, Parsed,
};
use super::yaml_text::{block_continues, normalize_uses, scan_block_line};

mod detail;

pub(crate) use detail::{step_run_lines, step_uses_lines, StepRun, StepUse};

pub(super) fn hits_from_lines(source: &str, action: bool, composite: bool) -> Vec<Hit> {
    let lines: Vec<&str> = source.lines().collect();
    let mut scanner = Scanner::from_lines(&lines, action, composite, false);
    scanner.scan();
    scanner.hits
}

pub(super) struct Scanner<'a> {
    lines: &'a [&'a str],
    action: bool,
    composite: bool,
    pub(super) locate: bool,
    pub(super) collect_runs: bool,
    frames: Vec<Frame>,
    pub(super) open_quote: Option<(BlockRole, char)>,
    pub(super) uses: Option<String>,
    pub(super) run_lines: Vec<(usize, String)>,
    pub(super) script_lines: Vec<(usize, String)>,
    pub(super) hits: Vec<Hit>,
    pub(super) job: String,
    pub(super) step: usize,
    pub(super) uses_line: Option<usize>,
    pub(super) item_line: Option<usize>,
    pub(super) located: Vec<StepUse>,
    pub(super) runs: Vec<StepRun>,
}

impl<'a> Scanner<'a> {
    pub(super) fn from_lines(
        lines: &'a [&'a str],
        action: bool,
        composite: bool,
        locate: bool,
    ) -> Self {
        Self {
            lines,
            action,
            composite,
            locate,
            collect_runs: false,
            frames: Vec::new(),
            open_quote: None,
            uses: None,
            run_lines: Vec::new(),
            script_lines: Vec::new(),
            hits: Vec::new(),
            job: String::new(),
            step: 0,
            uses_line: None,
            item_line: None,
            located: Vec::new(),
            runs: Vec::new(),
        }
    }

    pub(super) fn scan(&mut self) {
        let mut index = 0;
        while index < self.lines.len() {
            let line = self.lines[index];
            let line_no = index + 1;
            if let Some((role, quote)) = self.open_quote {
                detail::push_line(self, role, line_no, line);
                if !detail::ends_quote(line, quote) {
                    self.open_quote = None;
                }
                index += 1;
                continue;
            }
            if let Some(Kind::Block(role, literal)) = self.frames.last().map(|frame| frame.kind) {
                let indent = self.frames.last().map(|frame| frame.indent).unwrap_or(0);
                if block_continues(line, indent) {
                    if scan_block_line(literal, line) {
                        detail::push_line(self, role, line_no, line);
                    }
                    index += 1;
                    continue;
                }
                self.pop_frame();
                continue;
            }
            if let Some(parsed) = parse_line(line) {
                self.apply(parsed, index);
            } else if self.locate {
                self.note_unparsed_item(line, line_no);
            }
            index += 1;
        }
        self.finish();
    }

    fn apply(&mut self, parsed: Parsed, index: usize) {
        self.pop_until(parsed.scope, parsed.is_item);
        if parsed.is_item && self.parent() == Some(Kind::Steps) {
            detail::begin_step(self, index + 1);
            self.frames.push(Frame {
                indent: parsed.scope,
                kind: Kind::Step,
            });
        }
        let Some(key) = parsed.key.as_deref() else {
            return;
        };
        let parent = self.parent();
        if key == "uses" && parent == Some(Kind::Step) {
            self.uses = Some(normalize_uses(&parsed.value));
            detail::note_uses(self, index + 1);
            // A folded or literal `uses` value is still this step's action reference.
            if !parsed.block {
                return;
            }
        }
        let role = script_role(key, parent);
        if parsed.block {
            self.frames.push(Frame {
                indent: parsed.key_indent,
                kind: Kind::Block(role.unwrap_or(BlockRole::Other), true),
            });
            return;
        }
        if parsed.nested {
            if role.is_some() && !next_is_child_key(self.lines, index, parsed.key_indent) {
                self.frames.push(Frame {
                    indent: parsed.key_indent,
                    kind: Kind::Block(role.unwrap_or(BlockRole::Other), false),
                });
            } else {
                let kind = classify_key(key, parent, self.action, self.composite);
                if kind == Kind::Job {
                    detail::begin_job(self, key);
                }
                self.frames.push(Frame {
                    indent: parsed.key_indent,
                    kind,
                });
            }
            return;
        }
        if let Some(role) = role {
            detail::record_inline(self, role, index + 1, &parsed.value);
        }
    }

    pub(super) fn pop_until(&mut self, scope: usize, is_item: bool) {
        while let Some(frame) = self.frames.last() {
            if frame.indent > scope
                || (frame.indent == scope && !(is_item && frame.kind == Kind::Steps))
            {
                self.pop_frame();
                continue;
            }
            break;
        }
    }

    fn pop_frame(&mut self) {
        // `pop_until` and `finish` only call this while `frames` is non-empty.
        if let Some(frame) = self.frames.pop() {
            if frame.kind == Kind::Step {
                detail::end_step(self);
                detail::flush_step(self);
            }
        }
    }

    fn finish(&mut self) {
        self.open_quote = None;
        while !self.frames.is_empty() {
            self.pop_frame();
        }
    }

    pub(super) fn parent(&self) -> Option<Kind> {
        self.frames.last().map(|frame| frame.kind)
    }

    // serde counts every sequence entry. Values this parser skips still occupy a step.
    fn note_unparsed_item(&mut self, line: &str, line_no: usize) {
        let Some(scope) = detail::item_scope(line) else {
            return;
        };
        self.pop_until(scope, true);
        if self.parent() == Some(Kind::Steps) {
            detail::begin_step(self, line_no);
            self.frames.push(Frame {
                indent: scope,
                kind: Kind::Step,
            });
        }
    }
}
