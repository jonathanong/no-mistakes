use super::expressions::expressions;
use super::hit::Hit;
use super::yaml_parse::{
    classify_key, next_is_child_key, parse_line, script_role, BlockRole, Frame, Kind, Parsed,
};
use super::yaml_text::{
    block_continues, ends_inside_quote, is_github_script, normalize_uses, scan_block_line,
};

pub(super) fn hits_from_lines(source: &str, action: bool, composite: bool) -> Vec<Hit> {
    let lines: Vec<&str> = source.lines().collect();
    let mut scanner = Scanner {
        lines: &lines,
        action,
        composite,
        frames: Vec::new(),
        open_quote: None,
        uses: None,
        run_lines: Vec::new(),
        script_lines: Vec::new(),
        hits: Vec::new(),
    };
    scanner.scan();
    scanner.hits
}

struct Scanner<'a> {
    lines: &'a [&'a str],
    action: bool,
    composite: bool,
    frames: Vec<Frame>,
    open_quote: Option<(BlockRole, char)>,
    uses: Option<String>,
    run_lines: Vec<(usize, String)>,
    script_lines: Vec<(usize, String)>,
    hits: Vec<Hit>,
}

impl Scanner<'_> {
    fn scan(&mut self) {
        let mut index = 0;
        while index < self.lines.len() {
            let line = self.lines[index];
            let line_no = index + 1;
            if let Some((role, quote)) = self.open_quote {
                self.push_line(role, line_no, line);
                if !ends_inside_quote(line, true, quote) {
                    self.open_quote = None;
                }
                index += 1;
                continue;
            }
            if let Some(Kind::Block(role, literal)) = self.frames.last().map(|frame| frame.kind) {
                let indent = self.frames.last().map(|frame| frame.indent).unwrap_or(0);
                if block_continues(line, indent) {
                    if scan_block_line(literal, line) {
                        self.push_line(role, line_no, line);
                    }
                    index += 1;
                    continue;
                }
                self.pop_frame();
                continue;
            }
            if let Some(parsed) = parse_line(line) {
                self.apply(parsed, index);
            }
            index += 1;
        }
        self.finish();
    }

    fn apply(&mut self, parsed: Parsed, index: usize) {
        self.pop_until(parsed.scope, parsed.is_item);
        if parsed.is_item && self.parent() == Some(Kind::Steps) {
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
            return;
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
                self.frames.push(Frame {
                    indent: parsed.key_indent,
                    kind: classify_key(key, parent, self.action, self.composite),
                });
            }
            return;
        }
        if let Some(role) = role {
            self.record_inline(role, index + 1, &parsed.value);
        }
    }

    fn record_inline(&mut self, role: BlockRole, line_no: usize, value: &str) {
        self.push_line(role, line_no, value);
        let trimmed = value.trim();
        let bytes = trimmed.as_bytes();
        // Callers only reach here with a non-empty scalar, so the first byte exists.
        if bytes
            .first()
            .is_some_and(|byte| *byte == b'"' || *byte == b'\'')
        {
            let quote = bytes[0] as char;
            if ends_inside_quote(trimmed, false, quote) {
                self.open_quote = Some((role, quote));
            }
        }
    }

    fn push_line(&mut self, role: BlockRole, line_no: usize, text: &str) {
        let slot = match role {
            BlockRole::Run => &mut self.run_lines,
            BlockRole::Script => &mut self.script_lines,
            BlockRole::Other => return,
        };
        slot.push((line_no, text.to_string()));
    }

    fn pop_until(&mut self, scope: usize, is_item: bool) {
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
                self.flush_step();
            }
        }
    }

    fn finish(&mut self) {
        self.open_quote = None;
        while !self.frames.is_empty() {
            self.pop_frame();
        }
    }

    fn flush_step(&mut self) {
        let uses = self.uses.take();
        let run_lines = std::mem::take(&mut self.run_lines);
        let script_lines = std::mem::take(&mut self.script_lines);
        for (line, text) in run_lines {
            for expr in expressions(&text) {
                self.hits.push(Hit {
                    line,
                    expr,
                    script: false,
                });
            }
        }
        if uses.as_deref().is_some_and(is_github_script) {
            for (line, text) in script_lines {
                for expr in expressions(&text) {
                    self.hits.push(Hit {
                        line,
                        expr,
                        script: true,
                    });
                }
            }
        }
    }

    fn parent(&self) -> Option<Kind> {
        self.frames.last().map(|frame| frame.kind)
    }
}
