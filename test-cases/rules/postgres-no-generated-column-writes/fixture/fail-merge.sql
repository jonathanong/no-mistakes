MERGE INTO items t
USING (SELECT $1::uuid AS id) s
ON t.id = s.id
WHEN MATCHED THEN UPDATE SET created_at = now()
WHEN NOT MATCHED THEN INSERT (id, created_at) VALUES (s.id, now());
