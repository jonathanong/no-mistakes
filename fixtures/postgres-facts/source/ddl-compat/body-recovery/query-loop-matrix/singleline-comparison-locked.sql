-- Keep the same query across the line-break, ANY, and row-lock axes.
CREATE FUNCTION query_loop_matrix(ids bigint[]) RETURNS void LANGUAGE plpgsql AS $body$
DECLARE
    selected_id bigint;
BEGIN
    FOR selected_id IN SELECT abs(t.id) FROM accounts AS t WHERE t.id > 0 ORDER BY t.id FOR NO KEY UPDATE LOOP
        PERFORM abs(selected_id);
    END LOOP;
END
$body$;
CREATE TABLE following_query_loop (id bigint);
