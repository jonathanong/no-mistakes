use super::calls::per_item_callees;
use super::carry::Carry;
use super::notes::{record, record_inline_body, record_loop_line, record_saved};
use super::statement::{close_unbraced, delimiter_delta, params_of, params_of_loop, Unbraced};
use super::syntax::{
    bare_header, block_header, brace_block, brace_header, end_count, indent_header, opener_count,
};
use super::{close_brace_loops, push, Header, Scan};

struct Pending {
    header: Header,
    calls: Vec<String>,
    line: u32,
    params: Vec<String>,
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
        if trimmed.starts_with('{') {
            if let Some(waiting) = pending.take() {
                push(
                    &mut stack,
                    &mut scan,
                    waiting.header,
                    depth,
                    false,
                    waiting.params,
                );
                record_saved(&mut scan, &stack, waiting.line, &waiting.calls);
            }
            record(&mut scan, &stack, line, line_no, &mut carry, false);
        } else if let Some(header) = brace_header(line) {
            pending = None;
            let is_loop = matches!(header, Header::Loop(_));
            let params = params_of(&header, line);
            push(&mut stack, &mut scan, header, depth, false, params);
            if is_loop {
                record_loop_line(&mut scan, &stack, line, line_no, &mut carry);
            } else {
                record_inline_body(&mut scan, &stack, line, line_no, false);
                carry.note(line, line_no);
            }
        } else if let Some(header) = bare_header(line) {
            let calls = match &header {
                Header::Loop(construct) => per_item_callees(line, construct),
                Header::Function(_) => Vec::new(),
            };
            let params = params_of(&header, line);
            pending = Some(Pending {
                params,
                header,
                calls,
                line: line_no,
            });
            carry.note(line, line_no);
        } else if let Some(waiting) = pending
            .take()
            .filter(|waiting| matches!(waiting.header, Header::Loop(_)))
        {
            push(
                &mut stack,
                &mut scan,
                waiting.header,
                depth,
                false,
                waiting.params,
            );
            record_saved(&mut scan, &stack, waiting.line, &waiting.calls);
            record(&mut scan, &stack, line, line_no, &mut carry, false);
            unbraced = Some(Unbraced {
                depth,
                delimiters: 0,
            });
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
        close_unbraced(&mut stack, &mut unbraced, depth, line);
    }
    scan
}

pub(super) fn scan_indent(source: &str) -> Scan {
    let mut scan = Scan::default();
    let mut stack: Vec<super::Frame> = Vec::new();
    let mut carry = Carry::default();
    let mut delimiters = 0i32;
    for (index, line) in source.lines().enumerate() {
        if line.trim().is_empty() {
            carry.clear();
            continue;
        }
        let line_no = index as u32 + 1;
        if delimiters == 0 {
            let indent = line
                .chars()
                .take_while(|ch| *ch == ' ' || *ch == '\t')
                .count();
            while stack.last().is_some_and(|frame| frame.open() >= indent) {
                stack.pop();
            }
            if let Some(header) = indent_header(line) {
                let is_loop = matches!(header, Header::Loop(_));
                push(
                    &mut stack,
                    &mut scan,
                    header,
                    indent,
                    false,
                    params_of_loop(is_loop, line),
                );
                if is_loop {
                    record_loop_line(&mut scan, &stack, line, line_no, &mut carry);
                } else {
                    record_inline_body(&mut scan, &stack, line, line_no, false);
                    carry.note(line, line_no);
                }
            } else {
                record(&mut scan, &stack, line, line_no, &mut carry, false);
            }
        } else {
            record(&mut scan, &stack, line, line_no, &mut carry, false);
        }
        delimiters = (delimiters + delimiter_delta(line)).max(0);
    }
    scan
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
            let is_loop = matches!(header, Header::Loop(_));
            push(
                &mut stack,
                &mut scan,
                header,
                depth,
                brace,
                params_of_loop(is_loop, line),
            );
            opened_brace = brace && is_loop;
            if is_loop {
                record_loop_line(&mut scan, &stack, line, line_no, &mut carry);
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
