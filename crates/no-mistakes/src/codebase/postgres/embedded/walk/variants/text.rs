use super::{origins, placeholders, Recovered};

impl Recovered {
    pub(super) fn append_sql(&mut self, suffix: &Self) {
        if self.sql.is_empty() && self.positions.is_empty() {
            self.line = suffix.line;
        }
        let offset = placeholders::count_placeholders(&self.sql);
        self.invocations
            .append(&suffix.invocations, &suffix.sql, offset, self.sql.len());
        self.origins.extend(origins::rewrite(
            &suffix.sql,
            &suffix.origins,
            offset,
            false,
        ));
        let mut positions = suffix.positions.clone();
        super::super::resolve::append::positions::renumber(&mut positions, &suffix.sql, offset);
        super::super::resolve::append::positions::append(
            &mut self.positions,
            &self.sql,
            self.line,
            suffix.line,
            &positions,
        );
        if self.first_append_offset.is_none() {
            self.first_append_offset = suffix.first_append_offset.map(|at| {
                self.sql.len()
                    + placeholders::renumber_placeholders(&suffix.sql[..at], offset).len()
            });
        }
        self.sql
            .push_str(&placeholders::renumber_placeholders(&suffix.sql, offset));
        self.merge_append_sites(suffix);
        self.fragment |= suffix.fragment;
        self.enumerated |= suffix.enumerated;
        self.variants_truncated |= suffix.variants_truncated;
        for choice in &suffix.choices {
            if !self.choices.contains(choice) {
                self.choices.push(*choice);
            }
        }
    }
}

#[cfg(test)]
mod tests;
