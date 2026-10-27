//! Token walk over a JSON document: resolves object keys and array indexes at
//! their own depth, independent of line layout or spacing.

struct Walker<'a> {
    bytes: &'a [u8],
    pos: usize,
    /// Byte offsets of every newline, for offset-to-line lookups.
    newlines: Vec<usize>,
}

/// The line of the deepest segment resolved and whether the walk parsed cleanly.
pub(super) fn json_key_line(source: &str, path: &[String]) -> (usize, bool) {
    let mut walker = Walker {
        bytes: source.as_bytes(),
        pos: 0,
        newlines: source.match_indices('\n').map(|(at, _)| at).collect(),
    };
    let mut found = 1;
    let parsed = walker.walk(path, &mut found).is_some();
    (found, parsed)
}

impl Walker<'_> {
    fn line(&self, offset: usize) -> usize {
        self.newlines.partition_point(|&at| at < offset) + 1
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn skip_ws(&mut self) {
        while let Some(byte) = self.peek() {
            let rest = &self.bytes[self.pos..];
            if byte.is_ascii_whitespace() {
                self.pos += 1;
            } else if rest.starts_with(b"//") {
                self.pos += rest.iter().position(|&b| b == b'\n').unwrap_or(rest.len());
            } else if rest.starts_with(b"/*") {
                let close = rest.windows(2).position(|pair| pair == b"*/");
                self.pos += close.map_or(rest.len(), |at| at + 2);
            } else {
                break;
            }
        }
    }

    /// Consumes a quoted string and returns its raw content range.
    fn string(&mut self) -> Option<(usize, usize)> {
        let quote = self.peek().filter(|b| matches!(b, b'"' | b'\''))?;
        let start = self.pos + 1;
        let mut at = start;
        while let Some(&byte) = self.bytes.get(at) {
            if byte == b'\\' {
                at += 1;
            } else if byte == quote {
                self.pos = at + 1;
                return Some((start, at));
            }
            at += 1;
        }
        None
    }

    fn skip_value(&mut self) -> Option<()> {
        self.skip_ws();
        match self.peek()? {
            b'"' | b'\'' => self.string().map(drop),
            b'{' | b'[' => self.skip_container(),
            _ => {
                while self
                    .peek()
                    .is_some_and(|b| !b.is_ascii_whitespace() && !matches!(b, b',' | b'}' | b']'))
                {
                    self.pos += 1;
                }
                Some(())
            }
        }
    }

    fn skip_container(&mut self) -> Option<()> {
        let mut depth = 0usize;
        loop {
            self.skip_ws();
            match self.peek()? {
                b'"' | b'\'' => {
                    self.string()?;
                    continue;
                }
                b'{' | b'[' => depth += 1,
                b'}' | b']' => depth -= 1,
                _ => {}
            }
            self.pos += 1;
            if depth == 0 {
                return Some(());
            }
        }
    }

    /// Descends the value at the cursor along `path`, recording the line of
    /// each matched segment in `found`. `None` means the text was malformed.
    fn walk(&mut self, path: &[String], found: &mut usize) -> Option<()> {
        self.skip_ws();
        match self.peek()? {
            b'{' => self.walk_object(path, found),
            b'[' => self.walk_array(path, found),
            _ => Some(()),
        }
    }

    fn walk_object(&mut self, path: &[String], found: &mut usize) -> Option<()> {
        self.pos += 1;
        loop {
            self.skip_ws();
            match self.peek()? {
                b'}' => return Some(()),
                b',' => {
                    self.pos += 1;
                    continue;
                }
                _ => {}
            }
            let key_at = self.pos;
            let (start, end) = self.string()?;
            self.skip_ws();
            (self.peek()? == b':').then_some(())?;
            self.pos += 1;
            self.skip_ws();
            if &self.bytes[start..end] == path[0].as_bytes() {
                *found = self.line(key_at);
                return self.descend(path, found);
            }
            self.skip_value()?;
        }
    }

    fn walk_array(&mut self, path: &[String], found: &mut usize) -> Option<()> {
        let Ok(nth) = path[0].parse::<usize>() else {
            return Some(());
        };
        self.pos += 1;
        let mut index = 0;
        loop {
            self.skip_ws();
            match self.peek()? {
                b']' => return Some(()),
                b',' => {
                    self.pos += 1;
                    continue;
                }
                _ => {}
            }
            if index == nth {
                *found = self.line(self.pos);
                return self.descend(path, found);
            }
            self.skip_value()?;
            index += 1;
        }
    }

    fn descend(&mut self, path: &[String], found: &mut usize) -> Option<()> {
        match path.get(1..).filter(|rest| !rest.is_empty()) {
            Some(rest) => self.walk(rest, found),
            None => Some(()),
        }
    }
}
