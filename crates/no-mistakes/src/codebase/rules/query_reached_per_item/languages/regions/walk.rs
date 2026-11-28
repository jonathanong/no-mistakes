use super::calls::{enclosing_loop_line_callees, per_item_callees};
use super::carry::Carry;
use super::lex::same_line_loop_tail;
use super::notes::{record, record_inline_body, record_loop_line, record_saved, record_tail};
use super::params::parameter_names;
use super::statement::{close_unbraced, params_of, params_of_loop, release_armed, Unbraced};
use super::syntax::{
    bare_header, block_header, brace_block_offset, brace_header, end_count, opener_count,
};
use super::{push, step_brace_loops, Header, Scan};

pub(super) use super::indent::scan_indent;

struct Pending {
    header: Header,
    calls: Vec<String>,
    line: u32,
    signature: String,
}

pub(super) fn scan_brace(source: &str) -> Scan {
    let mut scan = Scan::default();
    let mut stack: Vec<super::Frame> = Vec::new();
    let mut depth = 0usize;
    let mut carry = Carry::default();
    let mut pending: Option<Pending> = None;
    let mut unbraced: Option<Unbraced> = None;
    for (index, line) in source.lines().enumerate() {
        let line_no = index as u32 + 1;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            carry.clear();
            continue;
        }
        release_armed(&mut stack, &mut unbraced, trimmed);
        let mut close_line = line;
        if trimmed.starts_with('{') {
            if let Some(waiting) = pending.take() {
                let params = pending_params(&waiting);
                push(&mut stack, &mut scan, waiting.header, depth, None, params);
                record_saved(&mut scan, &stack, waiting.line, &waiting.calls);
            }
            record(&mut scan, &stack, line, line_no, &mut carry, false);
        } else if matches!(
            pending,
            Some(Pending {
                header: Header::Function(_),
                ..
            })
        ) && (trimmed.starts_with(')') || brace_header(line).is_none())
        {
            if let Some((before, tail)) = line.split_once('{') {
                if let Some(mut waiting) = pending.take() {
                    waiting.signature.push(' ');
                    waiting.signature.push_str(before.trim());
                    let params = pending_params(&waiting);
                    push(&mut stack, &mut scan, waiting.header, depth, None, params);
                    record_tail(&mut scan, &stack, tail, line_no);
                    carry.note(line, line_no);
                }
            } else if trimmed.ends_with(';') || trimmed.starts_with('}') {
                pending = None;
            } else if let Some(waiting) = pending.as_mut() {
                waiting.signature.push(' ');
                waiting.signature.push_str(trimmed);
            }
        } else if let Some(header) = brace_header(line) {
            pending = None;
            let loop_construct = match header {
                Header::Loop(construct) => Some(construct),
                Header::Function(_) => None,
            };
            let params = params_of(&header, line);
            push(&mut stack, &mut scan, header, depth, None, params);
            if let Some(construct) = loop_construct {
                record_loop_line(&mut scan, &stack, construct, line, line_no, &mut carry);
            } else {
                record_inline_body(&mut scan, &stack, line, line_no, false);
                carry.note(line, line_no);
            }
        } else if let Some(header) = bare_header(line) {
            if let Header::Loop(construct) = header {
                if let Some(tail) = same_line_loop_tail(line, construct) {
                    pending = None;
                    push(&mut stack, &mut scan, header, depth, None, Vec::new());
                    record_loop_line(&mut scan, &stack, construct, line, line_no, &mut carry);
                    unbraced = Some(Unbraced::at(depth));
                    close_line = tail;
                } else {
                    let enclosing = enclosing_loop_line_callees(line, construct);
                    record_saved(&mut scan, &stack, line_no, &enclosing);
                    pending = Some(Pending {
                        header,
                        calls: per_item_callees(line, construct),
                        line: line_no,
                        signature: String::new(),
                    });
                    carry.note(line, line_no);
                }
            } else {
                pending = Some(Pending {
                    header,
                    calls: Vec::new(),
                    line: line_no,
                    signature: line.to_string(),
                });
                carry.note(line, line_no);
            }
        } else if let Some(waiting) = pending.take() {
            push(
                &mut stack,
                &mut scan,
                waiting.header,
                depth,
                None,
                Vec::new(),
            );
            record_saved(&mut scan, &stack, waiting.line, &waiting.calls);
            record(&mut scan, &stack, line, line_no, &mut carry, false);
            unbraced = Some(Unbraced::at(depth));
        } else {
            record(&mut scan, &stack, line, line_no, &mut carry, false);
        }
        for ch in line.chars() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth = depth.saturating_sub(1);
                    super::pop_to(&mut stack, depth);
                }
                _ => {}
            }
        }
        close_unbraced(&stack, &mut unbraced, depth, close_line);
    }
    scan
}

fn pending_params(waiting: &Pending) -> Vec<String> {
    match waiting.header {
        Header::Function(_) => parameter_names(&waiting.signature),
        Header::Loop(_) => Vec::new(),
    }
}

pub(super) fn scan_block(source: &str) -> Scan {
    let mut scan = Scan::default();
    let mut stack: Vec<super::Frame> = Vec::new();
    let mut depth = 0usize;
    let mut brace_depth = 0usize;
    let mut carry = Carry::default();
    for (index, line) in source.lines().enumerate() {
        if line.trim().is_empty() {
            carry.clear();
            continue;
        }
        let line_no = index as u32 + 1;
        let mut brace_tail = line;
        if let Some(header) = block_header(line) {
            let loop_construct = match header {
                Header::Loop(construct) => Some(construct),
                Header::Function(_) => None,
            };
            let brace_open = loop_construct
                .and_then(|_| brace_block_offset(line))
                .map(|at| {
                    step_brace_loops(&mut stack, &mut depth, &mut brace_depth, &line[..at]);
                    brace_tail = &line[at..];
                    brace_depth
                });
            push(
                &mut stack,
                &mut scan,
                header,
                depth,
                brace_open,
                params_of_loop(loop_construct.is_some(), line),
            );
            if let Some(construct) = loop_construct {
                record_loop_line(&mut scan, &stack, construct, line, line_no, &mut carry);
            } else {
                record_inline_body(&mut scan, &stack, line, line_no, true);
                carry.note(line, line_no);
            }
        } else {
            record(&mut scan, &stack, line, line_no, &mut carry, true);
        }
        depth += opener_count(line);
        for _ in 0..end_count(line) {
            depth = depth.saturating_sub(1);
            super::pop_to(&mut stack, depth);
        }
        step_brace_loops(&mut stack, &mut depth, &mut brace_depth, brace_tail);
    }
    scan
}
