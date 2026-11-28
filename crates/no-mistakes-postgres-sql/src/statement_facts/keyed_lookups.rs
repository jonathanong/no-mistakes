/// One duplicate keyed lookup of the same relation within a single statement scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlDuplicateKeyedLookupFact {
    /// Canonical target relation name (e.g. `people` or `public.people`).
    pub relation: String,
    /// Human-readable normalized key summary (e.g. `id = n.owner_id`).
    pub key_display: String,
    /// 1-based line of the first lookup (rebased to physical source line for embedded SQL).
    pub first_line: usize,
    /// 1-based SQL column of the first lookup.
    pub first_column: usize,
    /// 1-based line of the duplicate lookup (rebased to physical source line for embedded SQL).
    pub line: usize,
    /// 1-based SQL column of the duplicate lookup.
    pub column: usize,
}
