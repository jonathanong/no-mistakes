-- The derived UNION ALL source is capped even though TABLE is its right arm.
SELECT 1 FROM (SELECT NULL UNION ALL TABLE accounts LIMIT 1) d;
