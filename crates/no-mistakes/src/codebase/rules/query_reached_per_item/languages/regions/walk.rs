use super::calls::{enclosing_loop_line_callees, per_item_callees};
use super::carry::Carry;
use super::notes::{
    record, record_after_brace, record_inline_body, record_loop_line, record_saved,
};
use super::params::parameter_names;
use super::statement::{close_unbraced, params_of, params_of_loop, release_armed, Unbraced};
use super::syntax::{
    bare_header, block_header, brace_block, brace_header, end_count, opener_count,
};
use super::{close_brace_loops, push, Header, Scan};

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
        if trimmed.starts_with('{') {
            if let Some(waiting) = pending.take() {
                let params = pending_params(&waiting);
                push(&mut stack, &mut scan, waiting.header, depth, false, params);
                record_saved(&mut scan, &stack, waiting.line, &waiting.calls);
            }
            record(&mut scan, &stack, line, line_no, &mut carry, false);
        } else if matches!(pending, Some(Pending { header: Header::Function(_), .. }))
            && (trimmed.starts_with(')') || brace_header(line).is_none())
        {
            if let Some((before, _)) = line.split_once('{') {
                if let Some(mut waiting) = pending.take() {
                    waiting.signature.push(' ');
                    waiting.signature.push_str(before.trim());
                    let params = pending_params(&waiting);
                    push(&mut stack, &mut scan, waiting.header, depth, false, params);
                    record_after_brace(&mut scan, &stack, line, line_no);
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
            push(&mut stack, &mut scan, header, depth, false, params);
            if let Some(construct) = loop_construct {
                record_loop_line(&mut scan, &stack, construct, line, line_no, &mut carry);
            } else {
                record_inline_body(&mut scan, &stack, line, line_no, false);
                carry.note(line, line_no);
            }
        } else if let Some(header) = bare_header(line) {
            let (calls, signature) = match &header {
                Header::Loop(construct) => {
                    let enclosing = enclosing_loop_line_callees(line, construct);
                    record_saved(&mut scan, &stack, line_no, &enclosing);
                    (per_item_callees(line, construct), String::new())
                }
                Header::Function(_) => (Vec::new(), line.to_string()),
            };
            pending = Some(Pending {
                header,
                calls,
                line: line_no,
                signature,
            });
            carry.note(line, line_no);
        } else if let Some(waiting) = pending.take() {
            push(&mut stack, &mut scan, waiting.header, depth, false, Vec::new());
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
        close_unbraced(&stack, &mut unbraced, depth, line);
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
    let mut carry = Carry::default();
    for (index, line) in source.lines().enumerate() {
        if line.trim().is_empty() {
            carry.clear();
            continue;
        }
        let line_no = index as u32 + 1;
        let mut opened_brace = false;
        if let Some(header) = block_header(line) {
            let brace = brace_block(line);
            let loop_construct = match header {
                Header::Loop(construct) => Some(construct),
                Header::Function(_) => None,
            };
            push(
                &mut stack,
                &mut scan,
                header,
                depth,
                brace,
                params_of_loop(loop_construct.is_some(), line),
            );
            opened_brace = brace && loop_construct.is_some();
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
        close_brace_loops(&mut stack, &mut depth, line, opened_brace);
    }
    scan
}
