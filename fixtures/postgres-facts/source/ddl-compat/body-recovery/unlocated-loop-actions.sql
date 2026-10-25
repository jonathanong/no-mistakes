-- The unlocated fallback must project only the action, not SQL in a format string.
FOR rec IN EXECUTE format('SELECT abs(id) AT TIME ZONE ''UTC'' FROM %I', 'accounts') LOOP PERFORM abs(5);
FOR current_id IN SELECT abs(id) FROM accounts ORDER BY id FOR NO KEY UPDATE LOOP PERFORM abs(6);
LOOP PERFORM abs(7);
