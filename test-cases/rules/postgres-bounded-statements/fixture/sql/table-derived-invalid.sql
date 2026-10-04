-- An incomplete TABLE right arm must remain a parse error.
SELECT 1 FROM (SELECT NULL UNION ALL TABLE) d;
