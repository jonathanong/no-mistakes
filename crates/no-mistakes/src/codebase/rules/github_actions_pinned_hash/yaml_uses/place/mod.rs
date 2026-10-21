/// Block and flow nesting for job and step `uses` keys.
///
/// Mirrors `collect_in`: a `uses` string counts only when its mapping is a job
/// or a step. A root sequence is a step list. `jobs` / `steps` open those
/// places; every other key opens `Root`, so a matrix `include` entry is not a
/// step.
mod line;

use line::{
    anchor_only, child_place, is_block_key, split_line, without_leading_anchor, Level, Place, Split,
};

pub(super) struct PlaceStack {
    levels: Vec<Level>,
    pending: Option<Place>,
    pending_indent: usize,
    key_indent: usize,
    line_indent: usize,
}

impl Default for PlaceStack {
    fn default() -> Self {
        Self {
            levels: vec![Level::root_map()],
            pending: None,
            pending_indent: 0,
            key_indent: 0,
            line_indent: 0,
        }
    }
}

impl PlaceStack {
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(super) fn begin_line(&mut self, line: &str) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return;
        }
        self.line_indent = line.chars().take_while(|ch| ch.is_whitespace()).count();
        let split = split_line(line);
        let pop_at = if split.sequence {
            split.dash_indent
        } else {
            split.key_indent
        };
        self.pop_to(pop_at, split.sequence);
        let pending = self.pending.take();
        let pending_indent = self.pending_indent;
        if split.sequence {
            self.open_sequence(&split, pending, pending_indent);
        } else if let Some(place) = pending {
            if split.key_indent > pending_indent && is_block_key(split.body) {
                self.push(place, true, false, split.key_indent, false, false);
            }
        }
        self.key_indent = split.key_indent;
    }

    pub(super) fn note_key(&mut self, key: &str) -> bool {
        let place = self
            .levels
            .last()
            .map(|level| level.place)
            .unwrap_or(Place::Root);
        self.pending = Some(child_place(place, key));
        self.pending_indent = self.key_indent;
        key == "uses" && matches!(place, Place::Job | Place::Step)
    }

    /// A `uses` key on the root mapping stays a regex hit. A nested one does not.
    pub(super) fn nested(&self) -> bool {
        self.levels
            .last()
            .is_some_and(|level| level.flow || !level.root)
    }

    pub(super) fn open_flow(&mut self, sequence: bool) {
        if let Some(place) = self.pending.take() {
            self.push_flow(place, sequence);
            return;
        }
        if let Some(parent) = self.levels.last().filter(|level| !level.mapping).copied() {
            let place = if parent.items_are_steps {
                Place::Step
            } else {
                parent.place
            };
            self.push_flow(place, sequence);
            return;
        }
        if sequence && self.only_root_map() {
            self.levels[0] = Level::root_sequence(0, true);
        }
    }

    pub(super) fn close_flow(&mut self) {
        if self
            .levels
            .last()
            .is_some_and(|level| level.flow && !level.root)
        {
            self.levels.pop();
        } else if self.levels.last().is_some_and(|level| level.flow) {
            self.levels[0] = Level::root_map();
        }
        self.pending = None;
    }

    pub(super) fn clear_pending(&mut self) {
        self.pending = None;
    }

    fn open_sequence(&mut self, split: &Split<'_>, pending: Option<Place>, pending_indent: usize) {
        if self.only_root_map() && pending.is_none() {
            self.levels[0] = Level::root_sequence(split.dash_indent, false);
        } else if let Some(place) = pending {
            // `steps:` then `- uses` in the same column is still that sequence.
            if split.dash_indent >= pending_indent {
                self.push(
                    place,
                    false,
                    false,
                    split.dash_indent,
                    place == Place::Steps,
                    false,
                );
            }
        }
        // `- &checkout name:` is a step key. The anchor is not part of the key.
        if is_block_key(without_leading_anchor(split.body)) {
            self.push_sequence_item(self.item_place(), split.key_indent);
        } else if self.in_sequence()
            && (split.body.is_empty() || split.body.starts_with('#') || anchor_only(split.body))
        {
            // A bare `-`, `- # note`, or `- &name` opens the item. The next key stays inside it.
            self.push_sequence_item(self.item_place(), split.dash_indent);
        }
    }

    fn push_flow(&mut self, place: Place, sequence: bool) {
        let steps = sequence && place == Place::Steps;
        self.push(place, !sequence, true, 0, steps, false);
    }

    fn push_sequence_item(&mut self, place: Place, indent: usize) {
        self.push(place, true, false, indent, false, false);
        if let Some(top) = self.levels.last_mut() {
            top.sequence_item = true;
        }
    }

    fn push(
        &mut self,
        place: Place,
        mapping: bool,
        flow: bool,
        indent: usize,
        steps: bool,
        root: bool,
    ) {
        self.levels.push(Level {
            place,
            mapping,
            flow,
            indent,
            items_are_steps: steps,
            root,
            sequence_item: false,
        });
    }

    pub(super) fn line_indent(&self) -> usize {
        self.line_indent
    }

    pub(super) fn depth(&self) -> usize {
        self.levels.len()
    }

    /// The open mapping is the `jobs:` map, so `env` and `with` are job ids.
    pub(super) fn at_jobs(&self) -> bool {
        self.levels
            .last()
            .is_some_and(|level| level.place == Place::Jobs)
    }

    /// The current sequence item is a step. A root sequence counts.
    pub(super) fn item_is_step(&self) -> bool {
        self.item_place() == Place::Step
    }

    fn item_place(&self) -> Place {
        self.levels.last().map_or(Place::Step, |level| {
            if !level.mapping && level.items_are_steps {
                Place::Step
            } else {
                level.place
            }
        })
    }

    fn in_sequence(&self) -> bool {
        self.levels.last().is_some_and(|level| !level.mapping)
    }

    fn only_root_map(&self) -> bool {
        match self.levels.as_slice() {
            [level] => level.root && level.mapping && !level.flow,
            _ => false,
        }
    }

    fn pop_to(&mut self, indent: usize, sequence_item: bool) {
        while let Some(top) = self.levels.last() {
            if top.root || top.flow {
                break;
            }
            let previous_item =
                sequence_item && top.mapping && top.sequence_item && top.indent == indent;
            // A key in the sequence's column ends it (`steps:` / `-` / `timeout-minutes:`).
            let ended_sequence = !sequence_item && !top.mapping && top.indent == indent;
            if top.indent > indent || previous_item || ended_sequence {
                self.levels.pop();
                continue;
            }
            break;
        }
    }
}
