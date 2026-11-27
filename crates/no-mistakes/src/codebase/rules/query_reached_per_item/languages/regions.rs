use crate::codebase::lang_frontends::{mask_strings, strip_comments_keep_strings};

mod syntax;
pub(super) use syntax::split_separator;
use syntax::{block_header, brace_header, calls, end_count, indent_header, opener_count};

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
    Loop { construct: &'static str, open: usize },
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
    let prepared = mask_strings(&strip_comments_keep_strings(source));
    match style {
        Style::Brace => scan_brace(&prepared),
        Style::Indent => scan_indent(&prepared),
        Style::Block => scan_block(&prepared),
    }
}

fn scan_brace(source: &str) -> Scan {
    let mut scan = Scan::default();
    let mut stack = Vec::new();
    let mut depth = 0usize;
    for (index, line) in source.lines().enumerate() {
        let line_no = index as u32 + 1;
        if let Some(header) = brace_header(line) {
            push(&mut stack, &mut scan, header, depth);
        } else {
            record(&mut scan, &stack, line, line_no);
        }
        for ch in line.chars() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth = depth.saturating_sub(1);
                    pop_to(&mut stack, depth);
                }
                _ => {}
            }
        }
    }
    scan
}

fn scan_indent(source: &str) -> Scan {
    let mut scan = Scan::default();
    let mut stack: Vec<Frame> = Vec::new();
    for (index, line) in source.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let indent = line.chars().take_while(|ch| *ch == ' ' || *ch == '\t').count();
        while stack.last().is_some_and(|frame| indent <= frame.open()) {
            stack.pop();
        }
        if let Some(header) = indent_header(line) {
            push(&mut stack, &mut scan, header, indent);
            continue;
        }
        record(&mut scan, &stack, line, index as u32 + 1);
    }
    scan
}

fn scan_block(source: &str) -> Scan {
    let mut scan = Scan::default();
    let mut stack = Vec::new();
    let mut depth = 0usize;
    for (index, line) in source.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        if let Some(header) = block_header(line) {
            push(&mut stack, &mut scan, header, depth);
        } else {
            record(&mut scan, &stack, line, index as u32 + 1);
        }
        depth += opener_count(line);
        for _ in 0..end_count(line) {
            depth = depth.saturating_sub(1);
            pop_to(&mut stack, depth);
        }
    }
    scan
}

fn push(stack: &mut Vec<Frame>, scan: &mut Scan, header: Header, open: usize) {
    match header {
        Header::Function(name) => {
            scan.functions.push(name.clone());
            stack.push(Frame::Function { name, open });
        }
        Header::Loop(construct) => stack.push(Frame::Loop { construct, open }),
    }
}

fn pop_to(stack: &mut Vec<Frame>, depth: usize) {
    while stack.last().is_some_and(|frame| frame.open() >= depth) {
        stack.pop();
    }
}

fn record(scan: &mut Scan, stack: &[Frame], line: &str, line_no: u32) {
    let function = stack.iter().rev().find_map(|frame| match frame {
        Frame::Function { name, .. } => Some(name.clone()),
        Frame::Loop { .. } => None,
    });
    let construct = stack.iter().rev().find_map(|frame| match frame {
        Frame::Loop { construct, .. } => Some(*construct),
        Frame::Function { .. } => None,
    });
    for callee in calls(line) {
        if let Some(construct) = construct {
            scan.loops.push(LoopCall {
                line: line_no,
                callee,
                construct,
                function: function.clone().unwrap_or_else(|| "<module>".to_string()),
            });
        } else if let Some(function) = &function {
            scan.body_calls.push((function.clone(), callee));
        }
    }
}
