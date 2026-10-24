SELECT 1;
-- This unsupported DDL must be reported without hiding its parsed siblings.
ALTER TABLE accounts ADD COLUMN;
DISCARD ALL;
-- A second failed statement proves findings remain statement scoped.
FROBULATE accounts;
