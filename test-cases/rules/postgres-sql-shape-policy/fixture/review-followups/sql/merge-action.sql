MERGE INTO accounts AS target
USING (SELECT id FROM seed) AS source
ON target.id = source.id
WHEN MATCHED THEN
  UPDATE SET blocked = source.id NOT IN (SELECT id FROM bans)
WHEN MATCHED THEN
  UPDATE SET *
WHEN NOT MATCHED THEN
  INSERT (id, quiet)
  VALUES (source.id, (SELECT COUNT(*) FROM orders) = 0)
WHEN NOT MATCHED THEN
  INSERT *;
