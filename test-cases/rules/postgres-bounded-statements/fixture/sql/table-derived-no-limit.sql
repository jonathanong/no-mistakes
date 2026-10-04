-- A parenthesized unqualified TABLE arm without a trailing clause must parse.
SELECT 1 FROM (SELECT NULL UNION ALL TABLE accounts) d;
