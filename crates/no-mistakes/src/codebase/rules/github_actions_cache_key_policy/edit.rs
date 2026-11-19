use super::flow::{block_header, fill_with_text};
use super::scan::{Field, Step};
use super::text::yaml_plain;
use super::walk::Walker;

pub(super) enum Slot {
    Id,
    Uses,
    Run,
    Path,
    Key,
    Restore,
    Skip,
}

#[derive(Clone, Copy)]
pub(super) enum Seq {
    Path,
    Restore,
}

pub(super) struct Scalar {
    slot: Slot,
    line: usize,
    key_indent: usize,
    content_indent: Option<usize>,
    buf: String,
}

pub(super) fn finish_step(walker: &mut Walker) {
    finish_scalar(walker);
    walker.in_with = false;
    walker.seq = None;
}

pub(super) fn consume_scalar(walker: &mut Walker, raw: &str) -> bool {
    if walker.scalar.is_none() {
        return false;
    }
    if raw.trim().is_empty() {
        walker.scalar.as_mut().unwrap().buf.push('\n');
        return true;
    }
    let indent = super::flow::leading(raw);
    let key_indent = walker.scalar.as_ref().unwrap().key_indent;
    let content = walker.scalar.as_ref().unwrap().content_indent;
    let inside = content.is_none_or(|content| indent >= content)
        && (content.is_some() || indent > key_indent);
    if !inside {
        finish_scalar(walker);
        return false;
    }
    let scalar = walker.scalar.as_mut().unwrap();
    if scalar.content_indent.is_none() {
        scalar.content_indent = Some(indent);
    }
    let cut = scalar.content_indent.unwrap_or(indent).min(raw.len());
    scalar.buf.push_str(&raw[cut..]);
    scalar.buf.push('\n');
    true
}

pub(super) fn finish_scalar(walker: &mut Walker) {
    let Some(scalar) = walker.scalar.take() else {
        return;
    };
    store(
        walker,
        scalar.slot,
        scalar.line,
        scalar.buf.trim_end_matches('\n').to_string(),
    );
}

pub(super) fn consume_seq(walker: &mut Walker, line: usize, raw: &str) -> bool {
    let Some((seq, indent)) = walker.seq else {
        return false;
    };
    if let Some((item_indent, rest)) = super::flow::list_item(raw) {
        if item_indent > indent {
            append_seq(walker, seq, line, yaml_plain(rest));
            return true;
        }
    }
    walker.seq = None;
    false
}

pub(super) fn assign(walker: &mut Walker, slot: Slot, line: usize, indent: usize, value: &str) {
    if block_header(value) {
        start_scalar(walker, slot, line, indent);
        return;
    }
    if value.is_empty() && matches!(slot, Slot::Path | Slot::Restore) {
        let seq = if matches!(slot, Slot::Path) {
            Seq::Path
        } else {
            Seq::Restore
        };
        walker.seq = Some((seq, indent));
        return;
    }
    let text = if matches!(slot, Slot::Run) {
        value.to_string()
    } else {
        yaml_plain(value)
    };
    store(walker, slot, line, text);
}

pub(super) fn open_with(walker: &mut Walker, line: usize, indent: usize, value: &str) {
    if value.starts_with('{') {
        if let Some(step) = current(walker) {
            fill_with_text(step, line, value);
        }
        return;
    }
    if value.is_empty() {
        walker.in_with = true;
        walker.with_indent = indent;
    }
}

fn start_scalar(walker: &mut Walker, slot: Slot, line: usize, indent: usize) {
    finish_scalar(walker);
    walker.scalar = Some(Scalar {
        slot,
        line,
        key_indent: indent,
        content_indent: None,
        buf: String::new(),
    });
}

fn store(walker: &mut Walker, slot: Slot, line: usize, value: String) {
    let Some(step) = current(walker) else { return };
    let plain = yaml_plain(&value);
    match slot {
        Slot::Id => step.id = Some(plain),
        Slot::Uses => step.uses = Some(plain),
        Slot::Run => step.run = Some(if value.contains('\n') { value } else { plain }),
        Slot::Path => step.path = Some(Field { line, value: plain }),
        Slot::Key => step.key = Some(Field { line, value: plain }),
        Slot::Restore => step.restore_keys = Some(Field { line, value }),
        Slot::Skip => {}
    }
}

fn append_seq(walker: &mut Walker, seq: Seq, line: usize, text: String) {
    let Some(step) = current(walker) else { return };
    let field = match seq {
        Seq::Path => &mut step.path,
        Seq::Restore => &mut step.restore_keys,
    };
    match field {
        Some(existing) => {
            existing.value.push('\n');
            existing.value.push_str(&text);
        }
        None => *field = Some(Field { line, value: text }),
    }
}

fn current(walker: &mut Walker) -> Option<&mut Step> {
    walker.jobs.last_mut()?.steps.last_mut()
}
