MERGE INTO accounts AS target
USING unnest(ARRAY[(SELECT COUNT(*) FROM orders) > 0]) AS source(flag)
ON true
WHEN MATCHED THEN DELETE;

MERGE INTO accounts AS target
USING generate_series((SELECT COUNT(*) FROM orders) = 0, 1) AS source(n)
ON true
WHEN MATCHED THEN DELETE;
