-- Keep authored spellings for each compatibility-rewrite family.
FROBULATE 0o17;
FROBULATE FETCH FIRST (1 + 1) ROWS ONLY;
FROBULATE TABLE accounts;
FROBULATE GENERATED ALWAYS AS (1) VIRTUAL BAD;
FROBULATE ON DELETE SET NULL (a) BAD;
-- Unicode before a failure must not shift its source-byte slices.
SELECT 'π'; FROBULATE 0x1F;
-- The inserted grouping comma must not appear in an original-text head.
FROBULATE GROUP BY DISTINCT ROLLUP(x);
SELECT 1 GROUP BY DISTINCT ROLLUP(1); FROBULATE 0x1F;
