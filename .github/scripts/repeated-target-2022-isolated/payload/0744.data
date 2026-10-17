DO $body$ BEGIN
  EXECUTE (E'INSERT INTO t\n' || ($sql$VALUES($1) $sql$ || 'ON CONFLICT DO NOTHING')) USING $2, CAST(1 AS numeric);
  EXECUTE 'INSERT INTO t VALUES(1)' USING (SELECT 1);
  EXECUTE 42;
  EXECUTE command_head || 'VALUES(1)';
  EXECUTE 'INSERT INTO t VALUES(1)' INTO target_value;
  -- A failed expression must not consume the following statement's tokens.
  EXECUTE ;
  EXECUTE 'INSERT INTO t VALUES(2)';
  EXECUTE 'INSERT INTO t VALUES(3)' USING ;
  EXECUTE 'INSERT INTO t VALUES(4)';
END $body$;
