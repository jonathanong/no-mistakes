mod calls;
mod carry;
mod indent;
mod lex;
mod notes;
mod params;
mod prepare;
mod statement;
mod syntax;
mod walk;

pub(super) use calls::split_separator;

#[derive(Default)]
pub(super) struct Scan {
    pub(super) loops: Vec<LoopCall>,
    pub(super) functions: Vec<String>,
    pub(super) body_calls: Vec<(String, String)>,
    /// Parameter and enclosing-parameter names for each function, in source order.
    pub(super) shadows: Vec<(String, Vec<String>)>,
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

#[derive(Clone)]
enum Frame {
    Function {
        name: String,
        open: usize,
    },
    Loop {
        construct: &'static str,
        open: usize,
        brace: bool,
    },
}

impl Frame {
    fn open(&self) -> usize {
        match self {
            Self::Function { open, .. } | Self::Loop { open, .. } => *open,
        }
    }
}

fn style_for(extension: &str) -> Option<Style> {
    match extension {
        "py" => Some(Style::Indent),
        "rb" | "ex" | "exs" => Some(Style::Block),
        "go" | "rs" | "php" | "java" | "kt" | "dart" | "swift" | "cs" => Some(Style::Brace),
        _ => None,
    }
}

pub(super) fn scan(source: &str, extension: &str) -> Scan {
    let Some(style) = style_for(extension) else {
        return Scan::default();
    };
    let prepared = prepare::prepare(source, extension);
    match style {
        Style::Brace => walk::scan_brace(&prepared),
        Style::Indent => walk::scan_indent(&prepared),
        Style::Block => walk::scan_block(&prepared),
    }
}

fn push(
    stack: &mut Vec<Frame>,
    scan: &mut Scan,
    header: Header,
    open: usize,
    brace: bool,
    params: Vec<String>,
) {
    match header {
        Header::Function(name) => {
            let names = with_ancestors(scan, stack, params);
            scan.functions.push(name.clone());
            scan.shadows.push((name.clone(), names));
            stack.push(Frame::Function { name, open });
        }
        Header::Loop(construct) => stack.push(Frame::Loop {
            construct,
            open,
            brace,
        }),
    }
}

fn with_ancestors(scan: &Scan, stack: &[Frame], mut params: Vec<String>) -> Vec<String> {
    let Some(outer) = stack.iter().rev().find_map(|frame| match frame {
        Frame::Function { name, .. } => Some(name.as_str()),
        Frame::Loop { .. } => None,
    }) else {
        return params;
    };
    if let Some((_, bound)) = scan.shadows.iter().rev().find(|(name, _)| name == outer) {
        params.extend(bound.iter().cloned());
    }
    params
}

fn pop_to(stack: &mut Vec<Frame>, depth: usize) {
    while stack.last().is_some_and(|frame| frame.open() >= depth) {
        stack.pop();
    }
}

fn close_brace_loops(stack: &mut Vec<Frame>, depth: &mut usize, line: &str, opened_brace: bool) {
    let mut closers = line
        .matches('}')
        .count()
        .saturating_sub(line.matches('{').count());
    if opened_brace && line.contains('}') && closers == 0 {
        closers = 1;
    }
    while closers > 0 {
        let open = match stack.last() {
            Some(Frame::Loop {
                brace: true, open, ..
            }) => *open,
            _ => break,
        };
        *depth = open;
        stack.pop();
        closers -= 1;
    }
}
