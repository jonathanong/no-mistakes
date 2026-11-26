use crate::codebase::shell_words::{split_quoted_words, Whitespace};

mod lexer;
mod words;
use lexer::Connector;
use words::{for_limit, matching_command, read_variables};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum TestSignal {
    #[default]
    None,
    Direct,
    Negated,
    IfCond,
    IfDirectCond,
    IfNegatedCond,
    IfDirectThen,
    IfNegatedThen,
}

#[derive(Default)]
struct Loop {
    retries: Option<u64>,
    variables: Vec<String>,
    in_body: bool,
    matched_test: bool,
    prepared_input: bool,
    pipe_input: bool,
    uses_input: bool,
    retry_signal: bool,
    test_signal: TestSignal,
}

pub(super) fn rerun_loops(script: &str, commands: &[Vec<String>]) -> Vec<Option<u64>> {
    let mut loops = Vec::<Loop>::new();
    let mut found = Vec::new();
    for segment in lexer::segments(script) {
        let Some(words) = split_quoted_words(segment.text, Whitespace::Ascii) else {
            reset_signal(&mut loops);
            continue;
        };
        let mut words = words.as_slice();
        let Some(first) = words.first().map(String::as_str) else {
            continue;
        };
        match first {
            "for" => {
                reset_signal(&mut loops);
                let (variable, retries) = for_limit(words);
                loops.push(Loop {
                    retries,
                    variables: variable.into_iter().collect(),
                    ..Loop::default()
                });
                continue;
            }
            "while" | "until" => {
                reset_signal(&mut loops);
                loops.push(Loop {
                    variables: read_variables(&words[1..]),
                    matched_test: matching_command(&words[1..], commands).is_some(),
                    ..Loop::default()
                });
                continue;
            }
            "done" => {
                if let Some(frame) = loops.pop() {
                    if frame.matched_test
                        && frame.in_body
                        && (frame.retry_signal || !frame.uses_input)
                    {
                        found.push(frame.retries);
                    }
                }
                reset_signal(&mut loops);
                continue;
            }
            "do" => {
                let Some(frame) = loops.last_mut() else {
                    continue;
                };
                frame.in_body = true;
                words = &words[1..];
                if words.is_empty() {
                    continue;
                }
            }
            _ => {}
        }
        process_body_segment(
            &mut loops,
            segment.text,
            segment.preceded_by,
            words,
            commands,
        );
    }
    found
}

fn process_body_segment(
    loops: &mut [Loop],
    text: &str,
    preceded_by: Connector,
    mut words: &[String],
    commands: &[Vec<String>],
) {
    if preceded_by != Connector::Pipe {
        loops.iter_mut().for_each(|frame| frame.pipe_input = false);
    }
    let Some(last) = loops.last_mut().filter(|frame| frame.in_body) else {
        return;
    };
    loop {
        match words.first().map(String::as_str) {
            Some("if" | "elif") => last.test_signal = TestSignal::IfCond,
            Some("then") => {
                last.test_signal = match last.test_signal {
                    TestSignal::IfDirectCond => TestSignal::IfDirectThen,
                    TestSignal::IfNegatedCond => TestSignal::IfNegatedThen,
                    _ => TestSignal::None,
                };
            }
            Some("else") => {
                last.test_signal = match last.test_signal {
                    TestSignal::IfDirectThen => TestSignal::IfNegatedThen,
                    TestSignal::IfNegatedThen => TestSignal::IfDirectThen,
                    _ => TestSignal::None,
                };
            }
            Some("fi") => last.test_signal = TestSignal::None,
            _ => break,
        }
        words = &words[1..];
    }
    let Some(first) = words.first().map(String::as_str) else {
        return;
    };
    update_retry_signal(last, preceded_by, first);
    let in_if_cond = matches!(
        last.test_signal,
        TestSignal::IfCond | TestSignal::IfDirectCond | TestSignal::IfNegatedCond
    );
    if let Some(negated) = matching_command(words, commands) {
        last.matched_test = true;
        last.test_signal = match (in_if_cond, negated) {
            (true, false) => TestSignal::IfDirectCond,
            (true, true) => TestSignal::IfNegatedCond,
            (false, false) => TestSignal::Direct,
            (false, true) => TestSignal::Negated,
        };
        for frame in loops.iter_mut().filter(|frame| frame.in_body) {
            let expands = lexer::expands_any(text, &frame.variables);
            frame.uses_input |= frame.prepared_input || frame.pipe_input || expands;
            frame.pipe_input |= expands;
        }
    } else {
        let is_setup = matches!(first, "cd" | "pushd" | "export");
        for frame in loops.iter_mut().filter(|frame| frame.in_body) {
            let expands = lexer::expands_any(text, &frame.variables);
            frame.prepared_input |= is_setup && expands;
            frame.pipe_input |= expands;
        }
    }
}

fn update_retry_signal(frame: &mut Loop, preceded_by: Connector, first: &str) {
    match frame.test_signal {
        TestSignal::Direct => match (preceded_by, first) {
            (Connector::And, "break") | (Connector::Or, "continue") => {
                frame.retry_signal = true;
                frame.test_signal = TestSignal::None;
            }
            (Connector::And, _) => {}
            _ => frame.test_signal = TestSignal::None,
        },
        TestSignal::Negated => match (preceded_by, first) {
            (Connector::Or, "break") | (Connector::And, "continue") => {
                frame.retry_signal = true;
                frame.test_signal = TestSignal::None;
            }
            (Connector::And, _) => {}
            _ => frame.test_signal = TestSignal::None,
        },
        TestSignal::IfDirectThen if first == "break" => frame.retry_signal = true,
        TestSignal::IfNegatedThen if first == "continue" => frame.retry_signal = true,
        TestSignal::IfDirectCond | TestSignal::IfNegatedCond if preceded_by != Connector::And => {
            frame.test_signal = TestSignal::IfCond;
        }
        _ => {}
    }
}

fn reset_signal(loops: &mut [Loop]) {
    loops.iter_mut().for_each(|frame| frame.pipe_input = false);
    if let Some(frame) = loops.last_mut() {
        frame.test_signal = TestSignal::None;
    }
}
