-- Incomplete labels and missing THEN must never become recognized loop prefixes.
CREATE FUNCTION broken_label(ids bigint[]) RETURNS void LANGUAGE plpgsql AS $$
DECLARE selected_id bigint;
BEGIN
  <<loop_tasks> FOR selected_id IN SELECT id FROM accounts
    WHERE id = ANY(ids) FOR NO KEY UPDATE LOOP
    UPDATE accounts SET id = 1;
  END LOOP;
END;
$$;

CREATE FUNCTION missing_then(ids bigint[]) RETURNS void LANGUAGE plpgsql AS $$
DECLARE selected_id bigint;
BEGIN
  IF true
    FOR selected_id IN SELECT id FROM accounts
      WHERE id = ANY(ids) FOR NO KEY UPDATE LOOP
      UPDATE accounts SET id = 1;
    END LOOP;
  END IF;
END;
$$;
