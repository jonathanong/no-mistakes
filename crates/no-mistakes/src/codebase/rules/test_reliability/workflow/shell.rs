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
    let Some(last) = loops.last_mut().filter(|frame| frame.in_body) else {
        return;
    };
    let mut in_if_cond = false;
    loop {
        match words.first().map(String::as_str) {
            Some("if" | "elif") => {
                last.test_signal = TestSignal::None;
                in_if_cond = true;
                words = &words[1..];
            }
            Some("then") => {
                last.test_signal = match last.test_signal {
                    TestSignal::IfDirectCond => TestSignal::IfDirectThen,
                    TestSignal::IfNegatedCond => TestSignal::IfNegatedThen,
                    _ => TestSignal::None,
                };
                words = &words[1..];
            }
            Some("else" | "fi") => {
                last.test_signal = TestSignal::None;
                words = &words[1..];
            }
            _ => break,
        }
    }
    let Some(first) = words.first().map(String::as_str) else {
        return;
    };
    if !in_if_cond {
        update_retry_signal(last, preceded_by, first);
    }
    if let Some(negated) = matching_command(words, commands) {
        last.matched_test = true;
        last.test_signal = match (in_if_cond, negated) {
            (true, false) => TestSignal::IfDirectCond,
            (true, true) => TestSignal::IfNegatedCond,
            (false, false) => TestSignal::Direct,
            (false, true) => TestSignal::Negated,
        };
        for frame in loops.iter_mut().filter(|frame| frame.in_body) {
            frame.uses_input |= frame.prepared_input || lexer::expands_any(text, &frame.variables);
        }
    } else if matches!(first, "cd" | "pushd" | "export") {
        for frame in loops.iter_mut().filter(|frame| frame.in_body) {
            frame.prepared_input |= lexer::expands_any(text, &frame.variables);
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
        TestSignal::IfDirectCond | TestSignal::IfNegatedCond
            if preceded_by != Connector::And =>
        {
            frame.test_signal = TestSignal::None;
        }
        _ => {}
    }
}

fn reset_signal(loops: &mut [Loop]) {
    if let Some(frame) = loops.last_mut() {
        frame.test_signal = TestSignal::None;
    }
}
