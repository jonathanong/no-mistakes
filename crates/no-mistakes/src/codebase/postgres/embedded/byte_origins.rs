//! Canonical physical-byte origins for recovered SQL.
//!
//! A plain literal is one affine run. `0` is preserved; callers treat it as missing.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlByteOrigins {
    kind: Kind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Kind {
    Empty,
    Affine { base: u32, len: u32, step: u32 },
    Runs(Vec<Run>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Run {
    at: u32,
    origin: u32,
    len: u32,
    step: u32,
}

impl Default for SqlByteOrigins {
    fn default() -> Self {
        Self { kind: Kind::Empty }
    }
}

impl From<Vec<u32>> for SqlByteOrigins {
    fn from(values: Vec<u32>) -> Self {
        Self::compress(&values)
    }
}

impl SqlByteOrigins {
    pub fn empty() -> &'static Self {
        static EMPTY: SqlByteOrigins = SqlByteOrigins { kind: Kind::Empty };
        &EMPTY
    }

    pub fn get(&self, index: usize) -> Option<u32> {
        match &self.kind {
            Kind::Empty => None,
            Kind::Affine { base, len, step } => (index < *len as usize)
                .then_some(base.wrapping_add((index as u32).wrapping_mul(*step))),
            Kind::Runs(runs) => lookup(runs, index),
        }
    }

    pub fn at(&self, index: usize) -> u32 {
        self.get(index)
            .unwrap_or_else(|| panic!("sql byte origin {index} out of range"))
    }

    pub fn len(&self) -> usize {
        match &self.kind {
            Kind::Empty => 0,
            Kind::Affine { len, .. } => *len as usize,
            Kind::Runs(runs) => runs
                .last()
                .map(|run| run.at as usize + run.len as usize)
                .unwrap_or(0),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn first(&self) -> Option<u32> {
        self.get(0)
    }

    fn compress(values: &[u32]) -> Self {
        if values.is_empty() {
            return Self::default();
        }
        let mut runs = Vec::new();
        let mut index = 0;
        while index < values.len() {
            let origin = values[index];
            let (len, step) = match values.get(index + 1).copied() {
                None => (1, 0),
                Some(next) => {
                    let step = next.wrapping_sub(origin);
                    let mut len = 2;
                    while index + len < values.len()
                        && values[index + len].wrapping_sub(values[index + len - 1]) == step
                    {
                        len += 1;
                    }
                    (len, step)
                }
            };
            runs.push(Run {
                at: index as u32,
                origin,
                len: len as u32,
                step,
            });
            index += len;
        }
        if let [run] = runs.as_slice() {
            return Self {
                kind: Kind::Affine {
                    base: run.origin,
                    len: run.len,
                    step: run.step,
                },
            };
        }
        Self {
            kind: Kind::Runs(runs),
        }
    }
}

fn lookup(runs: &[Run], index: usize) -> Option<u32> {
    let mut found = None;
    for run in runs {
        let start = run.at as usize;
        let end = start + run.len as usize;
        if index >= start && index < end {
            let local = (index - start) as u32;
            found = Some(run.origin.wrapping_add(local.wrapping_mul(run.step)));
            break;
        }
    }
    found
}

#[cfg(test)]
#[path = "byte_origins_tests.rs"]
pub(crate) mod tests;
