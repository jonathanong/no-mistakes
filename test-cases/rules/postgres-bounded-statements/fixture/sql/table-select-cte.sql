-- A CTE arm names the local query, not a physical topics relation.
WITH topics AS (SELECT id FROM safe) SELECT id FROM safe UNION ALL TABLE topics;
