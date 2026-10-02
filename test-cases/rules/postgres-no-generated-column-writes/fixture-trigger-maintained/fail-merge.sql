MERGE INTO orders o USING incoming_orders i ON o.id = i.id
WHEN MATCHED THEN UPDATE SET status = i.status, updated_at = now();
