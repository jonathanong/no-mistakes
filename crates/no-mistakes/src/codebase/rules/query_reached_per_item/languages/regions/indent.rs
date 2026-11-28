use super::carry::Carry;
use super::notes::{record, record_inline_body, record_loop_line};
use super::statement::{params_of_loop, with_loop_held, IndentHold};
use super::syntax::indent_header;
use super::{push, Header, Scan};

pub(super) fn scan_indent(source: &str) -> Scan {
    let mut scan = Scan::default();
    let mut stack: Vec<super::Frame> = Vec::new();
    let mut carry = Carry::default();
    let mut hold = IndentHold::default();
    for (index, line) in source.lines().enumerate() {
        if line.trim().is_empty() {
            carry.clear();
            continue;
        }
        let line_no = index as u32 + 1;
        let opened_for = if hold.continued() {
            with_loop_held(&mut stack, hold.for_iterable(), |stack| {
                record(&mut scan, stack, line, line_no, &mut carry, false);
            });
            false
        } else {
            carry.clear();
            indent_at_column(&mut scan, &mut stack, &mut carry, line, line_no)
        };
        hold.advance(line, opened_for);
    }
    scan
}

fn indent_at_column(
    scan: &mut Scan,
    stack: &mut Vec<super::Frame>,
    carry: &mut Carry,
    line: &str,
    line_no: u32,
) -> bool {
    let indent = line
        .chars()
        .take_while(|ch| *ch == ' ' || *ch == '\t')
        .count();
    while stack.last().is_some_and(|frame| frame.open() >= indent) {
        stack.pop();
    }
    let Some(header) = indent_header(line) else {
        record(scan, stack, line, line_no, carry, false);
        return false;
    };
    let loop_construct = match header {
        Header::Loop(construct) => Some(construct),
        Header::Function(_) => None,
    };
    let opened_for = loop_construct == Some("for");
    push(
        stack,
        scan,
        header,
        indent,
        false,
        params_of_loop(loop_construct.is_some(), line),
    );
    if let Some(construct) = loop_construct {
        record_loop_line(scan, stack, construct, line, line_no, carry);
    } else {
        record_inline_body(scan, stack, line, line_no, false);
        carry.note(line, line_no);
    }
    opened_for
}
