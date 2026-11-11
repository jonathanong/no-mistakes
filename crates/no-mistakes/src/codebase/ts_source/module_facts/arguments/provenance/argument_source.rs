use std::cell::OnceCell;

pub(super) struct ArgumentSource<'a> {
    pub(super) text: &'a str,
    line_starts: OnceCell<Vec<u32>>,
}

impl<'a> ArgumentSource<'a> {
    pub(super) fn new(text: &'a str) -> Self {
        Self {
            text,
            line_starts: OnceCell::new(),
        }
    }

    pub(super) fn line(&self, offset: u32) -> u32 {
        let starts = self.line_starts.get_or_init(|| {
            if let Some(observer) = crate::diagnostics::current() {
                observer.increment("facts.argument_line_indexes", 1);
            }
            std::iter::once(0)
                .chain(
                    self.text
                        .bytes()
                        .enumerate()
                        .filter_map(|(index, byte)| (byte == b'\n').then_some((index + 1) as u32)),
                )
                .collect()
        });
        starts.partition_point(|start| *start <= offset) as u32
    }
}
