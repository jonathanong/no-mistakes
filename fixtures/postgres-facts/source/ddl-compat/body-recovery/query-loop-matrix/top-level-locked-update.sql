-- Loop compatibility is body scoped; this remains invalid top-level SQL.
    FOR selected_id IN SELECT abs(t.id) FROM accounts AS t
      WHERE t.id = ANY(ids)
      ORDER BY t.id
      FOR NO KEY UPDATE LOOP
        UPDATE accounts SET id = abs(id) WHERE id = selected_id;
    END LOOP;
