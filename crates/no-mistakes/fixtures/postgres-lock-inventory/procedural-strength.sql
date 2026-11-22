DO $body$
DECLARE
    selected_id bigint;
BEGIN
    NULL;
    SELECT id INTO selected_id FROM jobs FOR NO /* α /* SHARE */ KEY */ KEY UPDATE NOWAIT;
    SELECT id INTO selected_id FROM jobs FOR KEY -- UPDATE
        SHARE SKIP LOCKED;
    SELECT a.id INTO selected_id FROM jobs a JOIN jobs b ON a.id = b.id
        FOR NO KEY UPDATE OF a, /* second target */ b NOWAIT;
    SELECT id INTO selected_id FROM jobs FOR UPDATE;
    SELECT id INTO selected_id FROM jobs FOR /* KEY */ SHARE;
END
$body$;
SELECT id INTO selected_id FROM jobs FOR NO KEY UPDATE NOWAIT;
SELECT id INTO selected_id FROM jobs FOR KEY SHARE SKIP LOCKED;
SELECT a.id INTO selected_id FROM jobs a JOIN jobs b ON a.id = b.id
FOR NO KEY UPDATE OF a, b NOWAIT;
