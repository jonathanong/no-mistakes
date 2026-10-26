-- Keep the same query across the line-break, ANY, and row-lock axes.
CREATE FUNCTION query_loop_matrix(ids bigint[]) RETURNS void LANGUAGE plpgsql AS $body$
DECLARE
    selected_id bigint;
BEGIN
    IF 1 + THEN FOR selected_id IN SELECT abs(t.id) FROM accounts AS t
      WHERE t.id = ANY(ids)
      ORDER BY t.id
      FOR NO KEY UPDATE LOOP
        SELECT abs(id) FROM accounts WHERE id = selected_id;
    END LOOP; END IF;
END
$body$;
CREATE TABLE following_query_loop (id bigint);
