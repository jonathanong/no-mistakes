-- cspell:ignore uuidv
-- Loop utilities are classified but have no typed loop projection.
DO $$ BEGIN
  FOR i IN 1..2 LOOP
    CREATE TABLE bad (id uuid PRIMARY KEY DEFAULT uuidv7(), created_at timestamptz NOT NULL DEFAULT now());
  END LOOP;
END $$;
DO $$ BEGIN WHILE true LOOP CREATE TABLE while_child(id int); END LOOP; END $$;
DO $$ BEGIN FOREACH item IN ARRAY items LOOP CREATE TABLE foreach_child(id int); END LOOP; END $$;
DO $$ BEGIN
  FOR i IN 1..2 LOOP
    WHILE true LOOP CREATE TABLE nested_child(id int); END LOOP;
  END LOOP;
END $$;
DO $$ BEGIN
  IF true THEN
    FOR i IN 1..2 LOOP CREATE TABLE if_loop(id int); END LOOP;
  ELSE
    CREATE TABLE if_else(id int);
  END IF;
END $$;
DO $$ BEGIN
  FOR i IN 1..2 LOOP
    IF true THEN CREATE TABLE loop_if(id int);
    ELSE CREATE TABLE loop_else(id int); END IF;
  END LOOP;
END $$;
-- Parsed siblings must not hide an omitted loop utility.
DO $$ BEGIN
  CREATE TABLE before_loop(id int);
  FOR i IN 1..2 LOOP CREATE TABLE omitted_child(id int); END LOOP;
  CREATE TABLE after_loop(id int);
END $$;
DO $$ BEGIN
  <<retry>> FOR i IN 1..2 LOOP CREATE TABLE labeled_child(id int); END LOOP retry;
END $$;
DO $$ BEGIN
  <<retry>> WHILE true LOOP CREATE TABLE mismatched_child(id int); END LOOP other_label;
END $$;
DO $$ BEGIN FOR i IN 1..2 LOOP CREATE TABLE malformed_child(id int); END $$;
DO $$ BEGIN FOR i IN 1..2 LOOP INSERT INTO t(id) VALUES (i); END LOOP; END $$;
DO $$ BEGIN FOR i IN 1..2 LOOP EXECUTE 'CREATE TABLE literal_child(id int)'; END LOOP; END $$;
DO $$ BEGIN WHILE true LOOP EXECUTE command; END LOOP; END $$;
DO $$ BEGIN FOREACH item IN ARRAY items LOOP BOGUS utility_command; END LOOP; END $$;
DO $$ BEGIN LOOP NULL; END LOOP; END $$;
DO $$ BEGIN FOR row IN EXECUTE 'SELECT 1' LOOP RAISE NOTICE 'row'; END LOOP; END $$;
-- Typed IF branches remain complete, ordered, and globally located.
DO $$ BEGIN
  IF true THEN CREATE TABLE branch_first(id int);
  ELSIF false THEN CREATE TABLE branch_second(id int);
  ELSE CREATE TABLE branch_third(id int); END IF;
END $$;
-- Recognized utilities represented as typed Other remain compatible with #1733.
DO $$ BEGIN FOR i IN 1..2 LOOP LOCK TABLE t IN SHARE MODE; END LOOP; END $$;
DO $$ BEGIN WHILE LOOP CREATE TABLE empty_header(id int); END LOOP; END $$;
-- Literal EXECUTE children own decoded coordinates and their own completeness.
DO $$ BEGIN EXECUTE 'CREATE TABLE projected_literal(id int)'; END $$;
DO $$ BEGIN DO $inner$ BEGIN CREATE TABLE projected_nested(id int); END $inner$; END $$;
