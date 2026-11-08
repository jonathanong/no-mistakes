use super::placeholders;
use std::sync::Arc;

type InvocationRange = (usize, usize, u32);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Invocations(Option<Arc<Vec<InvocationRange>>>);

impl Invocations {
    fn iter(&self) -> impl Iterator<Item = &InvocationRange> {
        self.0.iter().flat_map(|ranges| ranges.iter())
    }

    fn ranges(&mut self) -> &mut Vec<InvocationRange> {
        Arc::make_mut(self.0.get_or_insert_with(|| Arc::new(Vec::new())))
    }
    pub(crate) fn record(&mut self, length: usize, site: u32) {
        if length > 0 {
            self.ranges().push((0, length, site));
        }
    }

    pub(crate) fn append(&mut self, suffix: &Self, sql: &str, binds: u32, prefix: usize) {
        if suffix.is_empty() {
            return;
        }
        self.ranges()
            .extend(suffix.iter().map(|&(start, end, site)| {
                let offset = |at| {
                    prefix
                        + if binds == 0 {
                            at
                        } else {
                            placeholders::renumber_placeholders(&sql[..at], binds).len()
                        }
                };
                (offset(start), offset(end), site)
            }));
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_none()
    }

    pub(crate) fn inherit(&mut self, caller: &Self, at: usize, length: usize) {
        for &(start, end, site) in caller.iter() {
            if start <= at && at < end {
                self.record(length, site);
            }
        }
    }

    pub(crate) fn slice(&self, range: std::ops::Range<usize>) -> Self {
        let ranges: Vec<_> = self
            .iter()
            .filter_map(|&(start, end, site)| {
                let (start, end) = (start.max(range.start), end.min(range.end));
                (start < end).then_some((
                    start.saturating_sub(range.start),
                    end.saturating_sub(range.start),
                    site,
                ))
            })
            .collect();
        if ranges.is_empty() {
            Self::default()
        } else {
            Self(Some(Arc::new(ranges)))
        }
    }

    pub(crate) fn remap(mut self, mut offset: impl FnMut(usize) -> usize) -> Self {
        if self.is_empty() {
            return self;
        }
        for (start, end, _) in self.ranges() {
            *start = offset(*start);
            *end = offset(*end);
        }
        self
    }

    pub(crate) fn publish(mut self, sql: &str) -> Self {
        if self.is_empty() || !sql.contains(placeholders::INTERNAL_PLACEHOLDER) {
            return self;
        }
        for (start, end, _) in self.ranges() {
            *start = placeholders::publish_placeholders(sql[..*start].to_owned()).len();
            *end = placeholders::publish_placeholders(sql[..*end].to_owned()).len();
        }
        self
    }

    pub(crate) fn at(&self, sql: &str, position: (usize, usize)) -> Vec<u32> {
        if self.is_empty() {
            return Vec::new();
        }
        let mut cursor = (1, 1);
        let offset = sql.char_indices().find_map(|(offset, character)| {
            let found = (cursor == position).then_some(offset);
            cursor = if character == '\n' {
                (cursor.0 + 1, 1)
            } else {
                (cursor.0, cursor.1 + 1)
            };
            found
        });
        self.iter()
            .filter(|&&(start, end, _)| offset.is_some_and(|at| start <= at && at < end))
            .map(|&(_, _, site)| site)
            .collect()
    }
}

#[cfg(test)]
#[path = "site_identity_tests.rs"]
mod tests;
