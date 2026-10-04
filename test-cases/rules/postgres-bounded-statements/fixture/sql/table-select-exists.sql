-- EXISTS still reads topics but does not expose its star projection.
SELECT 1 WHERE EXISTS (SELECT id FROM safe UNION ALL TABLE topics);
