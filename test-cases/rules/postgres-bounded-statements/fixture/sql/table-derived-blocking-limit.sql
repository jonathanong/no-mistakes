-- Blocking UNION must read its table before the outer LIMIT can cap output.
SELECT 1 FROM (SELECT NULL UNION TABLE accounts LIMIT 1) d;
