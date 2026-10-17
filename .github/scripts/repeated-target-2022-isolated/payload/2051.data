DO $$ BEGIN
  -- Runtime values must never be interpreted as literal commands.
  EXECUTE 'INSERT INTO t ' || command_tail;
  EXECUTE format('INSERT INTO %I VALUES(1)', table_name);
  EXECUTE command_text USING runtime_value;
END $$;
