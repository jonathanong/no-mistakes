use super::calls::{
    calls, enclosing_loop_line_callees, is_callee_byte, is_keyword, per_item_callees,
    split_separator,
};
use super::carry::Carry;
use super::params::signature_body;
use super::{Frame, LoopCall, Scan};

pub(super) fn record(
    scan: &mut Scan,
    stack: &[Frame],
    line: &str,
    line_no: u32,
    carry: &mut Carry,
    paren_free: bool,
) {
    let mut callees = carry.calls_on(line, line_no);
    if paren_free {
        callees.extend(
            paren_free_calls(line)
                .into_iter()
                .map(|callee| (line_no, callee)),
        );
    }
    record_callees(scan, stack, &callees);
}

pub(super) fn record_loop_line(
    scan: &mut Scan,
    stack: &[Frame],
    construct: &'static str,
    line: &str,
    line_no: u32,
    carry: &mut Carry,
) {
    let callees = per_item_callees(line, construct)
        .into_iter()
        .map(|callee| (line_no, callee))
        .collect::<Vec<_>>();
    carry.note(line, line_no);
    record_callees(scan, stack, &callees);
    let enclosing = enclosing_loop_line_callees(line, construct)
        .into_iter()
        .map(|callee| (line_no, callee))
        .collect::<Vec<_>>();
    if enclosing.is_empty() {
        return;
    }
    let outer = &stack[..stack.len().saturating_sub(1)];
    record_callees(scan, outer, &enclosing);
}

pub(super) fn record_tail(scan: &mut Scan, stack: &[Frame], tail: &str, line_no: u32) {
    let callees = calls(tail)
        .into_iter()
        .map(|callee| (line_no, callee))
        .collect::<Vec<_>>();
    record_callees(scan, stack, &callees);
}

pub(super) fn record_saved(scan: &mut Scan, stack: &[Frame], line_no: u32, callees: &[String]) {
    let pairs = callees
        .iter()
        .cloned()
        .map(|callee| (line_no, callee))
        .collect::<Vec<_>>();
    record_callees(scan, stack, &pairs);
}

pub(super) fn record_inline_body(
    scan: &mut Scan,
    stack: &[Frame],
    line: &str,
    line_no: u32,
    paren_free: bool,
) {
    let tail = signature_body(line);
    if tail.is_empty() {
        return;
    }
    let mut callees = calls(tail)
        .into_iter()
        .map(|callee| (line_no, callee))
        .collect::<Vec<_>>();
    if paren_free {
        callees.extend(
            paren_free_calls(tail)
                .into_iter()
                .map(|callee| (line_no, callee)),
        );
    }
    record_callees(scan, stack, &callees);
}

fn record_callees(scan: &mut Scan, stack: &[Frame], callees: &[(u32, String)]) {
    let (function, construct) = enclosing(stack);
    for (line_no, callee) in callees {
        if let Some(function) = &function {
            scan.body_calls.push((function.clone(), callee.clone()));
        }
        if let Some(construct) = construct {
            scan.loops.push(LoopCall {
                line: *line_no,
                callee: callee.clone(),
                construct,
                function: function.clone().unwrap_or_else(|| "<module>".to_string()),
            });
        }
    }
}

fn enclosing(stack: &[Frame]) -> (Option<String>, Option<&'static str>) {
    let mut function = None;
    let mut construct = None;
    for frame in stack.iter().rev() {
        match frame {
            Frame::Function { name, .. } => {
                function = Some(name.clone());
                break;
            }
            Frame::Loop {
                construct: label, ..
            } => {
                if construct.is_none() {
                    construct = Some(*label);
                }
            }
        }
    }
    (function, construct)
}

fn paren_free_calls(line: &str) -> Vec<String> {
    let bytes = line.as_bytes();
    let mut found = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if !(bytes[index].is_ascii_alphanumeric() || matches!(bytes[index], b'_' | b'$')) {
            index += 1;
            continue;
        }
        let start = index;
        while index < bytes.len() && is_callee_byte(bytes[index]) {
            index += 1;
        }
        let end = index;
        if end < bytes.len() && matches!(bytes[end], b'(' | b'.' | b':') {
            continue;
        }
        let mut cursor = end;
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        let argument = cursor < bytes.len()
            && (bytes[cursor].is_ascii_alphanumeric()
                || matches!(
                    bytes[cursor],
                    b'_' | b'$' | b'\'' | b'"' | b'@' | b'[' | b'{' | b'-'
                ));
        let zero_arg = cursor >= bytes.len() || matches!(bytes.get(cursor), Some(b';' | b'#'));
        if !argument && !zero_arg {
            continue;
        }
        let name = line[start..end].trim_start_matches(['.', ':', '\\', '>', '-']);
        if !name.is_empty() && !is_keyword(name) && split_separator(name).is_some() {
            found.push(name.to_string());
        }
        index = cursor;
    }
    found
}
