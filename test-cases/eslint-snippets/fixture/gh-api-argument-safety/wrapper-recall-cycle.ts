// ESLint-only: Oxlint 1.87 crashes on circular const initializers even with no enabled plugin rules.
const cycle = other;
const other = cycle;
cycle('unused', ['repos/O/R/labels?per_page=100']);
