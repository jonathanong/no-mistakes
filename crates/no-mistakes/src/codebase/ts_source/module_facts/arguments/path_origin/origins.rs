use super::{PathAnchor, PathOrigin};

impl PathOrigin {
    pub(crate) fn unknown() -> Self {
        Self {
            anchor: PathAnchor::Unknown,
            segments: Vec::new(),
            dynamic: true,
            env: None,
            getter: None,
            root_tails: Vec::new(),
            param: None,
            param_member: None,
            param_owner: None,
            owner_line: None,
            reassigned_at: None,
            alternatives: Vec::new(),
            param_default: None,
        }
    }

    pub(super) fn anchored(anchor: PathAnchor) -> Self {
        Self {
            anchor,
            segments: Vec::new(),
            dynamic: false,
            env: None,
            getter: None,
            root_tails: Vec::new(),
            param: None,
            param_member: None,
            param_owner: None,
            owner_line: None,
            reassigned_at: None,
            alternatives: Vec::new(),
            param_default: None,
        }
    }

    pub(super) fn literal(text: &str) -> Self {
        let absolute = text.starts_with(['/', '\\']) || text.as_bytes().get(1) == Some(&b':');
        let mut origin = Self {
            anchor: if absolute {
                PathAnchor::Absolute
            } else {
                PathAnchor::Relative
            },
            segments: Vec::new(),
            dynamic: false,
            env: None,
            getter: None,
            root_tails: Vec::new(),
            param: None,
            param_member: None,
            param_owner: None,
            owner_line: None,
            reassigned_at: None,
            alternatives: Vec::new(),
            param_default: None,
        };
        origin.extend(
            text.split(['/', '\\'])
                .filter(|part| !part.is_empty() && *part != ".")
                .take(32),
        );
        origin
    }

    pub(super) fn parent(mut self) -> Self {
        if !self.alternatives.is_empty() {
            return Self::alternatives(
                self.atoms()
                    .iter()
                    .map(|atom| atom.clone().parent())
                    .collect(),
            );
        }
        if self.segments.last().is_some_and(|last| last != "..") {
            self.segments.pop();
        } else if self.anchor == PathAnchor::Temp || self.param.is_some() || self.env.is_some() {
            self.segments.push("..".to_owned());
        }
        self
    }

    /// Add components, resolving `..` so `fixtures/../src` is not a fixture path.
    pub(super) fn extend<'s>(&mut self, parts: impl IntoIterator<Item = &'s str>) {
        for part in parts {
            if part == ".." && self.segments.last().is_some_and(|last| last != "..") {
                self.segments.pop();
            } else {
                self.segments.push(part.to_owned());
            }
        }
    }

    /// Append `tail` as trailing components of `self`.
    pub(super) fn append(self, tail: Self) -> Self {
        if self.alternatives.is_empty() && tail.alternatives.is_empty() {
            return self.append_single(tail);
        }
        Self::alternatives(
            self.atoms()
                .iter()
                .flat_map(|base| {
                    tail.atoms().iter().map(|tail| {
                        if tail.anchor == PathAnchor::Unknown {
                            tail.clone()
                        } else {
                            base.clone().append_single(tail.clone())
                        }
                    })
                })
                .collect(),
        )
    }

    fn append_single(mut self, tail: Self) -> Self {
        if (self.getter.is_some() || self.env.is_some() || self.anchor == PathAnchor::Temp)
            && (tail.dynamic || tail.anchor != PathAnchor::Relative)
        {
            self.root_tails.push(tail.clone());
        }
        self.dynamic |=
            tail.dynamic || !matches!(tail.anchor, PathAnchor::Relative | PathAnchor::Absolute);
        self.extend(tail.segments.iter().map(String::as_str));
        self
    }
}
