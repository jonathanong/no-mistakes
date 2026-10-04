-- A relation alias does not invent columns missing from every catalog-known child.
DELETE FROM orders o WHERE o.id IN (SELECT id FROM (currencies c CROSS JOIN currencies d) j LIMIT 1);
DELETE FROM orders o WHERE o.id IN (SELECT id FROM ((currencies c CROSS JOIN currencies d) j CROSS JOIN currencies e) k LIMIT 1);
DELETE FROM orders o WHERE o.id IN (SELECT id FROM (orders x CROSS JOIN currencies c) j LIMIT 1);
DELETE FROM orders o WHERE o.id IN (SELECT id FROM (currencies c CROSS JOIN (VALUES ($1::uuid)) v(id)) j LIMIT 1);
-- Explicit output aliases rename positions, so their existing conservative fallback stays intact.
DELETE FROM orders o WHERE o.id IN (SELECT id FROM (currencies c CROSS JOIN (VALUES ($1::uuid)) v(value)) j(code, id) LIMIT 1);
DELETE FROM orders o WHERE o.id IN (SELECT id FROM ((SELECT $1 AS id) x CROSS JOIN currencies c) j LIMIT 1);
