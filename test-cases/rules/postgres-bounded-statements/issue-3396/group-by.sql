-- Every post_id equals $1, so the grouped decision is one row and the update follows it.
WITH current_version AS (
 SELECT version.id AS version_id, post.id AS post_id
 FROM entries post JOIN entry_versions version ON version.post_id = post.id
 WHERE post.id = $1 FOR UPDATE OF post
), latest AS (
 SELECT DISTINCT ON (s.source) s.source
 FROM entry_signals s JOIN current_version ON current_version.version_id = s.version_id
 ORDER BY s.source, s.id DESC
), decision AS (
 SELECT current_version.post_id FROM current_version JOIN latest ON TRUE
 GROUP BY current_version.post_id HAVING count(*) = 2
), changed AS (
 SELECT decision.* FROM decision JOIN entries post ON post.id = decision.post_id
 WHERE post.approved_at IS NULL
), inserted AS (
 INSERT INTO entry_changes(id, post_id) SELECT $2, post_id FROM changed RETURNING post_id
)
UPDATE entries SET clearance_version = clearance_version + 1
FROM inserted WHERE entries.id = inserted.post_id;

-- Grouping a limited input stays bounded when another joined table would otherwise fan out.
WITH page AS (SELECT id FROM flows WHERE status = 'pending' ORDER BY id LIMIT 25)
SELECT page.id FROM page JOIN entry_signals signals ON TRUE GROUP BY page.id;

-- One caller value that is not a unique key is still one group.
SELECT flow.status FROM flows flow WHERE flow.status = 'pending' GROUP BY flow.status;

-- An unbounded group key keeps reporting.
SELECT signals.source FROM entry_signals signals GROUP BY signals.source;

-- One bounded key does not hide a second key from an unbounded table.
WITH page AS (SELECT id FROM flows WHERE status = 'pending' ORDER BY id LIMIT 25)
SELECT page.id, signals.source
FROM page
JOIN entry_signals signals ON TRUE
GROUP BY page.id, signals.source;

-- Grouping modifiers can add subtotal rows.
SELECT flow.status FROM flows flow WHERE flow.status = 'pending' GROUP BY ROLLUP(flow.status);

-- A set-returning projection can multiply the grouped rows.
SELECT flow.status, generate_series(1, 2)
FROM flows flow
WHERE flow.status = 'pending'
GROUP BY flow.status;

-- An integer GROUP BY literal is an output ordinal, not the pinned column.
SELECT flow.status FROM flows flow WHERE flow.status = 'pending' GROUP BY 1;

-- A set-returning ORDER BY can multiply the one group.
SELECT flow.status FROM flows flow
WHERE flow.status = 'pending'
GROUP BY flow.status
ORDER BY generate_series(1, 3);

-- An expression output has no source column, so the group does not fix the other input.
SELECT step.bucket
FROM (SELECT id + 1 AS bucket FROM flows WHERE status = 'pending') step
JOIN entry_signals signals ON TRUE
GROUP BY step.bucket;

-- Nine output renames do not carry one caller value into the group key.
WITH s0 AS (SELECT status AS k FROM flows WHERE status = 'pending'),
s1 AS (SELECT k AS k FROM s0),
s2 AS (SELECT k AS k FROM s1),
s3 AS (SELECT k AS k FROM s2),
s4 AS (SELECT k AS k FROM s3),
s5 AS (SELECT k AS k FROM s4),
s6 AS (SELECT k AS k FROM s5),
s7 AS (SELECT k AS k FROM s6),
s8 AS (SELECT k AS k FROM s7),
s9 AS (SELECT k AS k FROM s8)
SELECT s9.k FROM s9 JOIN entry_signals signals ON TRUE GROUP BY s9.k;

-- A list of two columns is not one peer value.
SELECT a.id FROM flows a, flows b, flows c WHERE a.id IN (b.id, c.id);

-- A positional alias list names the grouped output, not the inner select name.
SELECT d.status
FROM (SELECT status, id FROM flows WHERE status = 'pending') AS d(id, status)
JOIN entry_signals signals ON TRUE
GROUP BY d.status;

-- The same list in select order still groups the one caller value.
SELECT d.status
FROM (SELECT status, id FROM flows WHERE status = 'pending') AS d(status, id)
GROUP BY d.status;

-- A set-returning DISTINCT ON can multiply the one group.
SELECT DISTINCT ON (generate_series(1, 2)) flow.status
FROM flows flow
WHERE flow.status = 'pending'
GROUP BY flow.status;

-- DISTINCT ON of that same caller value is still one group.
SELECT DISTINCT ON (flow.status) flow.status
FROM flows flow
WHERE flow.status = 'pending'
GROUP BY flow.status;
