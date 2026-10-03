SELECT 0o_17;
SELECT 0b_10;
SELECT 0o_17 + 1;
SELECT 0 AS "o_17";
SELECT 0 o_17;
-- No space is intentional: a quoted alias stays separate from the preceding zero.
-- Radix repair must never merge this quoted word into a numeric literal.
SELECT 0"o_17";
