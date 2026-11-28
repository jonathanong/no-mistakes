mod calls;
mod prepare;
mod syntax;
mod walk;

pub(super) use calls::split_separator;
use calls::{calls, per_item_callees, trailing_callee};

#[derive(Default)]
pub(super) struct Scan {
    pub(super) loops: Vec<LoopCall>,
    pub(super) functions: Vec<String>,
    pub(super) body_calls: Vec<(String, String)>,
}

pub(super) struct LoopCall {
    pub(super) line: u32,
    pub(super) callee: String,
    pub(super) construct: &'static str,
    pub(super) function: String,
}

pub(super) enum Style {
    Brace,
    Indent,
    Block,
}

enum Header {
    Function(String),
    Loop(&'static str),
}

enum Frame {
    Function { name: String, open: usize },
    Loop { construct: &'static str, open: usize, brace: bool },
}

impl Frame {
    fn open(&self) -> usize {
        match self {
            Self::Function { open, .. } | Self::Loop { open, .. } => *open,
        }
    }
}

pub(super) fn style_for(extension: &str) -> Option<Style> {
    match extension {
        "py" => Some(Style::Indent),
        "rb" | "ex" | "exs" => Some(Style::Block),
        "go" | "rs" | "php" | "java" | "kt" | "dart" | "swift" | "cs" => Some(Style::Brace),
        _ => None,
    }
}

pub(super) fn scan(source: &str, style: Style) -> Scan {
    let prepared = prepare::prepare(source, matches!(style, Style::Indent));
    match style {
        Style::Brace => walk::scan_brace(&prepared),
        Style::Indent => walk::scan_indent(&prepared),
        Style::Block => walk::scan_block(&prepared),
    }
}

#[derive(Default)]
struct Carry {
    name: String,
}

impl Carry {
    fn clear(&mut self) {
        self.name.clear();
    }

    fn note(&mut self, line: &str) {
        self.name = if line.trim().is_empty() || line.contains('(') {
            String::new()
        } else {
            trailing_callee(line).unwrap_or("").to_string()
        };
    }

    fn calls_on(&mut self, line: &str) -> Vec<String> {
        if line.trim().is_empty() {
            self.clear();
            return Vec::new();
        }
        let found = if line.trim_start().starts_with('(') && !self.name.is_empty() {
            calls(&format!("{}{}", self.name, line))
        } else {
            calls(line)
        };
        self.note(line);
        found
    }
}

fn push(stack: &mut Vec<Frame>, scan: &mut Scan, header: Header, open: usize, brace: bool) {
    match header {
        Header::Function(name) => {
            scan.functions.push(name.clone());
            stack.push(Frame::Function { name, open });
        }
        Header::Loop(construct) => stack.push(Frame::Loop { construct, open, brace }),
    }
}

fn pop_to(stack: &mut Vec<Frame>, depth: usize) {
    while stack.last().is_some_and(|frame| frame.open() >= depth) {
        stack.pop();
    }
}

fn close_brace_loops(stack: &mut Vec<Frame>, depth: &mut usize, line: &str, opened_brace: bool) {
    let mut closers = line.matches('}').count().saturating_sub(line.matches('{').count());
    if opened_brace && line.contains('}') && closers == 0 {
        closers = 1;
    }
    while closers > 0 {
        let open = match stack.last() {
            Some(Frame::Loop { brace: true, open, .. }) => *open,
            _ => break,
        };
        *depth = open;
        stack.pop();
        closers -= 1;
    }
}

fn record(scan: &mut Scan, stack: &[Frame], line: &str, line_no: u32, carry: &mut Carry) {
    let callees = carry.calls_on(line);
    record_callees(scan, stack, line_no, &callees);
}

fn record_loop_line(scan: &mut Scan, stack: &[Frame], line: &str, line_no: u32, carry: &mut Carry) {
    let construct = match stack.last() {
        Some(Frame::Loop { construct, .. }) => *construct,
        _ => return,
    };
    let callees = per_item_callees(line, construct);
    carry.note(line);
    record_callees(scan, stack, line_no, &callees);
}

fn record_callees(scan: &mut Scan, stack: &[Frame], line_no: u32, callees: &[String]) {
    let (function, construct) = enclosing(stack);
    for callee in callees {
        if let Some(function) = &function {
            scan.body_calls.push((function.clone(), callee.clone()));
        }
        if let Some(construct) = construct {
            scan.loops.push(LoopCall {
                line: line_no,
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
            Frame::Loop { construct: label, .. } => {
                if construct.is_none() {
                    construct = Some(*label);
                }
            }
        }
    }
    (function, construct)
}
