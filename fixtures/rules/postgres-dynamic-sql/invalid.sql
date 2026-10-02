-- Tolerant recovery must not invent SQL from malformed control/format tokens.
DO $$ BEGIN
  THEN := 'CREATE TABLE invented_assignment (id int)';
  IF state = 1 THEN NULL; END IF;
  EXECUTE format 'CREATE TABLE invented_format (id int)' 'argument';
  EXECUTE pg_catalog bogus format ('CREATE TABLE invented_separator (id int)');
  EXECUTE pg_catalog.format 'CREATE TABLE invented_call (id int)' 'argument';
END $$;
