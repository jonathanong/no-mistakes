const RUNNER_PREFIXES: &[&str] = &[
    "pnpm", "npx", "npm", "yarn", "bunx", "bun", "uv", "bundle", "exec", "dlx", "run", "x",
];

pub(super) struct ShellRerunHit {
    pub(super) line: usize,
    pub(super) token: String,
    pub(super) command: String,
    pub(super) shape: &'static str,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Sep {
    #[default]
    Start,
    Or,
    Other,
}

struct Segment {
    words: Vec<String>,
    line: usize,
    preceded_by: Sep,
}

pub(super) fn find_shell_reruns(script: &str, test_commands: &[Vec<String>]) -> Vec<ShellRerunHit> {
    let mut loop_depth = 0usize;
    let mut prev_matched = false;
    let mut hits = Vec::new();
    for seg in split_shell_segments(script) {
        let slice = strip_control_keywords(&seg.words, &mut loop_depth);
        let matched = match_test_command(slice, test_commands);
        if let Some(cmd) = &matched {
            let shape = if loop_depth > 0 {
                Some("a shell loop")
            } else if seg.preceded_by == Sep::Or && prev_matched {
                Some("an `||` fallback rerun")
            } else {
                None
            };
            if let Some(shape) = shape {
                hits.push(ShellRerunHit {
                    line: seg.line,
                    token: slice.first().cloned().unwrap_or_default(),
                    command: cmd.clone(),
                    shape,
                });
            }
        }
        prev_matched = matched.is_some();
    }
    hits
}

fn strip_control_keywords<'a>(mut slice: &'a [String], loop_depth: &mut usize) -> &'a [String] {
    while let Some(first) = slice.first().map(String::as_str) {
        match first {
            "for" | "until" | "while" => *loop_depth += 1,
            "do" | "then" | "else" | "elif" | "!" => {}
            "done" => *loop_depth = loop_depth.saturating_sub(1),
            _ => break,
        }
        slice = &slice[1..];
    }
    slice
}

fn match_test_command(words: &[String], configured: &[Vec<String>]) -> Option<String> {
    let clean: Vec<&str> = words
        .iter()
        .map(String::as_str)
        .skip_while(|w| w.contains('='))
        .collect();
    if let Some(cmd) = configured.iter().find(|c| contains_subsequence(&clean, c)) {
        return Some(cmd.join(" "));
    }
    let first = clean
        .iter()
        .copied()
        .find(|w| !RUNNER_PREFIXES.contains(w))?;
    configured
        .iter()
        .find(|cmd| cmd.len() == 1 && cmd[0] == first)
        .map(|cmd| cmd[0].clone())
}

fn contains_subsequence(words: &[&str], needle: &[String]) -> bool {
    !needle.is_empty()
        && words.len() >= needle.len()
        && words
            .windows(needle.len())
            .any(|win| win.iter().zip(needle).all(|(a, b)| *a == b.as_str()))
}

#[derive(Default)]
struct Lexer {
    out: Vec<Segment>,
    words: Vec<String>,
    word: String,
    line: usize,
    seg_line: usize,
    next_sep: Sep,
    quote: Option<char>,
}

impl Lexer {
    fn flush_word(&mut self) {
        if !self.word.is_empty() {
            self.words.push(std::mem::take(&mut self.word));
        }
    }

    fn flush_segment(&mut self, sep: Sep) {
        self.flush_word();
        if !self.words.is_empty() {
            self.out.push(Segment {
                words: std::mem::take(&mut self.words),
                line: self.seg_line,
                preceded_by: self.next_sep,
            });
        }
        self.next_sep = sep;
    }

    fn push(&mut self, ch: char) {
        if self.words.is_empty() && self.word.is_empty() {
            self.seg_line = self.line;
        }
        self.word.push(ch);
    }

    fn escape(&mut self, chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
        match chars.next() {
            Some('\n') => self.line += 1,
            Some(esc) => self.push(esc),
            None => {}
        }
    }

    fn step_quoted(
        &mut self,
        q: char,
        ch: char,
        chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    ) {
        if ch == '\n' {
            self.line += 1;
        }
        if ch == q {
            self.quote = None;
        } else if q == '"' && ch == '\\' {
            self.escape(chars);
        } else {
            self.push(ch);
        }
    }
}

fn split_shell_segments(script: &str) -> Vec<Segment> {
    let mut lex = Lexer {
        line: 1,
        seg_line: 1,
        ..Lexer::default()
    };
    let mut chars = script.chars().peekable();
    while let Some(ch) = chars.next() {
        if let Some(q) = lex.quote {
            lex.step_quoted(q, ch, &mut chars);
            continue;
        }
        match ch {
            '\\' => lex.escape(&mut chars),
            '\'' | '"' => lex.quote = Some(ch),
            '#' if lex.word.is_empty() => {
                while chars.peek().is_some_and(|c| *c != '\n') {
                    chars.next();
                }
            }
            '\n' | ';' | '|' | '&' => {
                let doubled = matches!(ch, '|' | '&') && chars.peek() == Some(&ch);
                if doubled {
                    chars.next();
                }
                lex.flush_segment(if ch == '|' && doubled {
                    Sep::Or
                } else {
                    Sep::Other
                });
                if ch == '\n' {
                    lex.line += 1;
                }
            }
            c if c.is_whitespace() => lex.flush_word(),
            c => lex.push(c),
        }
    }
    lex.flush_segment(Sep::Other);
    lex.out
}
