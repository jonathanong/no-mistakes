-- Similar prefixes must not turn malformed SQL into recognized body controls.
PERFORM id FROM accounts FOR KEY SHARE FROBULATE accounts;
EXECUTE format('ALTER TABLE %I', item) FROBULATE accounts;
SELECT id INTO FROM accounts;
FOR row_record IN SELECT FROM LOOP NULL;
FOREACH item IN ARRAY ARRAY['accounts'] LOOP EXECUTE format('ALTER TABLE %I', item) FROBULATE accounts;
-- Runtime SQL and literal payloads retain their existing opaque/static paths.
EXECUTE runtime_sql;
EXECUTE 'FROBULATE accounts';
PERFORM 1 INTO selected_id;
