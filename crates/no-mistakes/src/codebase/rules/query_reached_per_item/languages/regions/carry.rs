use super::calls::{calls, carried_calls, trailing_callee};

#[derive(Default)]
pub(super) struct Carry {
    name: String,
    line: u32,
}

impl Carry {
    pub(super) fn clear(&mut self) {
        self.name.clear();
        self.line = 0;
    }

    pub(super) fn note(&mut self, line: &str, line_no: u32) {
        if line.trim().is_empty() {
            self.clear();
            return;
        }
        match trailing_callee(line) {
            Some(name) => {
                self.name = name.to_string();
                self.line = line_no;
            }
            None => self.clear(),
        }
    }

    pub(super) fn calls_on(&mut self, line: &str, line_no: u32) -> Vec<(u32, String)> {
        if line.trim().is_empty() {
            self.clear();
            return Vec::new();
        }
        let found = if line.trim_start().starts_with('(') && !self.name.is_empty() {
            carried_calls(
                &format!("{}{}", self.name, line),
                self.name.len(),
                self.line,
                line_no,
            )
        } else {
            calls(line)
                .into_iter()
                .map(|callee| (line_no, callee))
                .collect()
        };
        self.note(line, line_no);
        found
    }
}
