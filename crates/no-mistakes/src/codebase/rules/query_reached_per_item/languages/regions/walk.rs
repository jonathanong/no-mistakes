use super::calls::per_item_callees;
use super::syntax::{
    bare_header, block_header, brace_block, brace_header, end_count, indent_header, opener_count,
};
use super::{close_brace_loops, push, record, record_loop_line, Carry, Header, Scan};

struct Pending {
    header: Header,
    calls: Vec<String>,
    line: u32,
}

pub(super) fn scan_brace(source: &str) -> Scan {
    let mut scan = Scan::default();
    let mut stack: Vec<super::Frame> = Vec::new();
    let mut depth = 0usize;
    let mut carry = Carry::default();
    let mut pending: Option<Pending> = None;
    for (index, line) in source.lines().enumerate() {
        let line_no = index as u32 + 1;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            carry.clear();
            continue;
        }
        if trimmed.starts_with('{') {
            if let Some(waiting) = pending.take() {
                push(&mut stack, &mut scan, waiting.header, depth, false);
                super::record_callees(&mut scan, &stack, waiting.line, &waiting.calls);
            }
            record(&mut scan, &stack, line, line_no, &mut carry);
        } else if let Some(header) = brace_header(line) {
            pending = None;
            let is_loop = matches!(header, Header::Loop(_));
            push(&mut stack, &mut scan, header, depth, false);
            if is_loop {
                record_loop_line(&mut scan, &stack, line, line_no, &mut carry);
            } else {
                carry.note(line);
            }
        } else if let Some(header) = bare_header(line) {
            let calls = match &header {
                Header::Loop(construct) => per_item_callees(line, construct),
                Header::Function(_) => Vec::new(),
            };
            pending = Some(Pending { header, calls, line: line_no });
            carry.note(line);
        } else {
            pending = None;
            record(&mut scan, &stack, line, line_no, &mut carry);
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
            let indent = line.chars().take_while(|ch| *ch == ' ' || *ch == '\t').count();
            while stack.last().is_some_and(|frame| frame.open() >= indent) {
                stack.pop();
            }
            if let Some(header) = indent_header(line) {
                let is_loop = matches!(header, Header::Loop(_));
                push(&mut stack, &mut scan, header, indent, false);
                if is_loop {
                    record_loop_line(&mut scan, &stack, line, line_no, &mut carry);
                } else {
                    carry.note(line);
                }
            } else {
                record(&mut scan, &stack, line, line_no, &mut carry);
            }
        } else {
            record(&mut scan, &stack, line, line_no, &mut carry);
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
            push(&mut stack, &mut scan, header, depth, brace);
            opened_brace = brace && is_loop;
            if is_loop {
                record_loop_line(&mut scan, &stack, line, line_no, &mut carry);
            } else {
                carry.note(line);
            }
        } else {
            record(&mut scan, &stack, line, line_no, &mut carry);
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

fn delimiter_delta(line: &str) -> i32 {
    line.chars().fold(0, |delta, ch| match ch {
        '(' | '[' | '{' => delta + 1,
        ')' | ']' | '}' => delta - 1,
        _ => delta,
    })
}
