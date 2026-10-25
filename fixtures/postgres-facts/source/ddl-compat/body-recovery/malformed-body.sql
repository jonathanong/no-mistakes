SELECT abs(id) INTO STRICT FROM accounts;
SELECT abs(id) INTO STRICT first_id, FROM accounts;
FOR rec IN EXECUTE format('SELECT ''UTC'' FROM %I', unknown +) LOOP;
FOR rec IN SELECT id FROM accounts ORDER BY id FOR NO KEY LOOP;
