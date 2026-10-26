-- The first body statement shares the header's semicolon chunk.
CREATE FUNCTION query_loop_insert(ids bigint[]) RETURNS void LANGUAGE plpgsql AS $body$
DECLARE
    selected_id bigint;
BEGIN
    FOR selected_id IN
      SELECT abs(t.id) FROM accounts AS t
      WHERE t.id = ANY(ids)
      ORDER BY t.id
      FOR NO KEY UPDATE
    LOOP
        INSERT INTO accounts (id) VALUES (abs(selected_id));
    END LOOP;
END
$body$;
CREATE TABLE following_query_loop (id bigint);
