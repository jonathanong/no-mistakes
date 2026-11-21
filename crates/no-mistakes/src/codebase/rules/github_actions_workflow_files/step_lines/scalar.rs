use super::cursor::Walker;
use super::flow::{self, yaml_plain};
use super::{Field, Step};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Mode {
    Block,
    Plain,
    Quote,
}

pub(super) struct Scalar {
    pub name: String,
    pub line: usize,
    pub key_indent: usize,
    pub content_indent: Option<usize>,
    pub buf: String,
    pub mode: Mode,
}

pub(super) struct Seq {
    pub name: String,
    pub indent: usize,
    pub line: usize,
}

pub(super) fn upsert(
    step: &mut Step,
    name: &str,
    line: usize,
    value_line: usize,
    value: String,
    block_key: bool,
    append: bool,
) {
    if append {
        if let Some(field) = step
            .fields
            .iter_mut()
            .rev()
            .find(|field| field.name == name)
        {
            if !field.value.is_empty() {
                field.value.push('\n');
            }
            field.value.push_str(&value);
            return;
        }
    }
    step.fields.push(Field {
        name: name.to_string(),
        line,
        value_line,
        value,
        block_key,
    });
}

pub(super) fn finish_step(walker: &mut Walker) {
    finish_scalar(walker);
    walker.in_with = false;
    walker.seq = None;
}

pub(super) fn consume_scalar(walker: &mut Walker, raw: &str) -> bool {
    let Some(mode) = walker.scalar.as_ref().map(|scalar| scalar.mode) else {
        return false;
    };
    match mode {
        Mode::Block => super::cont::consume_block(walker, raw),
        Mode::Plain => super::cont::consume_plain(walker, raw),
        Mode::Quote => super::cont::consume_quote(walker, raw),
    }
}

pub(super) fn finish_scalar(walker: &mut Walker) {
    let Some(scalar) = walker.scalar.take() else {
        return;
    };
    let value = match scalar.mode {
        Mode::Quote => super::cont::unwrap_quotes(scalar.buf.trim_end_matches('\n')),
        Mode::Block | Mode::Plain => scalar.buf.trim_end_matches('\n').to_string(),
    };
    store(walker, &scalar.name, scalar.line, value, true);
}

pub(super) fn consume_seq(walker: &mut Walker, line: usize, raw: &str) -> bool {
    let Some(seq) = walker.seq.as_ref() else {
        return false;
    };
    let name = seq.name.clone();
    let key_line = seq.line;
    let indent = seq.indent;
    if let Some((item_indent, rest)) = flow::list_item(raw) {
        if item_indent > indent {
            append_seq(walker, &name, key_line, line, yaml_plain(rest));
            return true;
        }
    }
    walker.seq = None;
    false
}

pub(super) fn assign(walker: &mut Walker, name: &str, line: usize, indent: usize, value: &str) {
    if flow::block_header(value) {
        start_scalar(walker, name, line, indent, Mode::Block, String::new());
        return;
    }
    if value.is_empty() {
        if name == "path" || name == "restore-keys" {
            walker.seq = Some(Seq {
                name: name.to_string(),
                indent,
                line,
            });
            return;
        }
        store(walker, name, line, String::new(), true);
        return;
    }
    if value.starts_with(['{', '[']) {
        store(walker, name, line, value.to_string(), true);
        return;
    }
    if value.starts_with(['"', '\'']) {
        if super::cont::quote_closed(value) {
            store(walker, name, line, value.to_string(), true);
        } else {
            start_scalar(walker, name, line, indent, Mode::Quote, value.to_string());
        }
        return;
    }
    start_scalar(walker, name, line, indent, Mode::Plain, value.to_string());
}

pub(super) fn open_with(walker: &mut Walker, line: usize, indent: usize, value: &str) {
    if value.starts_with('{') {
        if let Some(step) = current(walker) {
            upsert(step, "with", line, line, String::new(), true, false);
            flow::fill_with_text(step, line, value);
        }
        return;
    }
    if value.is_empty() {
        store(walker, "with", line, String::new(), true);
        walker.in_with = true;
        walker.with_indent = indent;
    }
}

fn start_scalar(
    walker: &mut Walker,
    name: &str,
    line: usize,
    indent: usize,
    mode: Mode,
    buf: String,
) {
    finish_scalar(walker);
    walker.scalar = Some(Scalar {
        name: name.to_string(),
        line,
        key_indent: indent,
        content_indent: None,
        buf,
        mode,
    });
}

fn store(walker: &mut Walker, name: &str, line: usize, value: String, plain: bool) {
    let Some(step) = current(walker) else {
        return;
    };
    let value = present(name, value, plain);
    upsert(step, name, line, line, value, true, false);
}

fn present(name: &str, value: String, plain: bool) -> String {
    if !plain || (name == "run" && value.contains('\n')) {
        return value;
    }
    yaml_plain(&value)
}

fn append_seq(walker: &mut Walker, name: &str, key_line: usize, line: usize, text: String) {
    let Some(step) = current(walker) else {
        return;
    };
    upsert(step, name, key_line, line, text, true, true);
}

fn current(walker: &mut Walker) -> Option<&mut Step> {
    walker.jobs.last_mut()?.steps.last_mut()
}
