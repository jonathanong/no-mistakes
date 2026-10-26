use super::*;

impl Scope {
    pub(in super::super) fn with_table_tokens(tokens: TableTokenCursor) -> Self {
        Self {
            frame: Rc::default(),
            table_tokens: Some(Rc::new(RefCell::new(tokens))),
        }
    }

    pub(super) fn advance_table_tokens_to_right_arm(
        &self,
        left_start: sqlparser::tokenizer::Location,
    ) {
        if let Some(tokens) = &self.table_tokens {
            tokens.borrow_mut().advance_to_right_arm(left_start);
        }
    }

    pub(super) fn advance_table_tokens_to_from(
        &self,
        select_start: sqlparser::tokenizer::Location,
    ) {
        if let Some(tokens) = &self.table_tokens {
            tokens.borrow_mut().advance_to_from(select_start);
        }
    }

    pub(super) fn get(&self, name: &str) -> Option<&SqlBoundQuery> {
        let mut frame = self.frame.as_ref();
        loop {
            if let Some(bound) = frame.ctes.get(name) {
                return Some(bound.as_ref());
            }
            frame = frame.parent.as_deref()?;
        }
    }

    pub(super) fn names(&self) -> BTreeMap<String, Option<BTreeSet<String>>> {
        let mut frames = Vec::new();
        let mut frame = Some(self.frame.as_ref());
        while let Some(current) = frame {
            frames.push(current);
            frame = current.parent.as_deref();
        }
        let mut names = BTreeMap::new();
        for frame in frames.into_iter().rev() {
            names.extend(frame.columns.clone());
        }
        names
    }

    pub(super) fn child(&self) -> Self {
        Self {
            frame: Rc::new(ScopeFrame {
                parent: Some(Rc::clone(&self.frame)),
                ..ScopeFrame::default()
            }),
            table_tokens: self.table_tokens.clone(),
        }
    }

    pub(super) fn set_columns(&mut self, name: String, columns: Option<BTreeSet<String>>) {
        Rc::make_mut(&mut self.frame).columns.insert(name, columns);
    }

    pub(super) fn column_names(&self, name: &str) -> Option<&Option<BTreeSet<String>>> {
        let mut frame = self.frame.as_ref();
        loop {
            if let Some(columns) = frame.columns.get(name) {
                return Some(columns);
            }
            frame = frame.parent.as_deref()?;
        }
    }

    pub(super) fn fixed_counts(&self, name: &str) -> Option<&BTreeSet<String>> {
        let mut frame = self.frame.as_ref();
        loop {
            if frame.ctes.contains_key(name) {
                return frame.fixed_counts.get(name);
            }
            frame = frame.parent.as_deref()?;
        }
    }

    pub(super) fn insert_counts(&mut self, name: String, counts: BTreeSet<String>) {
        Rc::make_mut(&mut self.frame)
            .fixed_counts
            .insert(name, counts);
    }

    pub(super) fn insert(&mut self, name: String, bound: SqlBoundQuery) {
        // Opaque recursive placeholders retain the metadata visible before replacement.
        let columns = self.column_names(&name).cloned().unwrap_or_default();
        let frame = Rc::make_mut(&mut self.frame);
        frame.columns.entry(name.clone()).or_insert(columns);
        frame.ctes.insert(name, Rc::new(bound));
    }
}
