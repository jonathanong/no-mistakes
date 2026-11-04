use super::super::yaml_parse::{
    classify_key, next_is_child_key, script_role, BlockRole, Frame, Kind, Parsed,
};
use super::super::yaml_text::normalize_uses;
use super::{detail, Scanner};

impl Scanner<'_> {
    pub(super) fn apply(&mut self, parsed: Parsed, index: usize) {
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
            if !parsed.block {
                return;
            }
        }
        let role = script_role(key, parent);
        self.scripts.note_header(role, index + 1, &parsed.value);
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
            if self.scripts.collects_plain(&parsed.value) {
                self.frames.push(Frame {
                    indent: parsed.key_indent,
                    kind: Kind::Block(role, false),
                });
            }
        }
    }
}
