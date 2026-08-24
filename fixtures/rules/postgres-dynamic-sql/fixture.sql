SELECT 'ALTER TABLE ignored ADD COLUMN nope text';
SELECT 'DO $$ BEGIN CREATE TABLE fake_routine (id uuid); END $$';
DO E'BEGIN\nCREATE TABLE escaped_do_body (id uuid);\nEND';
DO U&'BEGIN
CREATE TABLE unicode_do_body (id uuid);
END';
DO 'BEGIN
CREATE TABLE plain_do_body (id uuid);
END';
DO 'BEGIN
'
'CREATE TABLE concatenated_do_body (id uuid);
END';
DO 'BEGIN
RAISE NOTICE ''CREATE TABLE inert_do_body (id uuid)'';
END';
DO $$
DECLARE
  ddl text := 'ALTER TABLE initial_posts ADD COLUMN ignored text';
  semicolon_ddl text;
BEGIN
  ddl := 'ALTER TABLE posts ADD COLUMN status text NOT NULL DEFAULT ''draft''';
  EXECUTE ddl;
  ddl := build_sql(column_name);
  EXECUTE ddl;
  EXECUTE format('CREATE TABLE %I (id uuid)', table_name);
  EXECUTE 'CREATE TABLE semicolon_sql (value text DEFAULT ''a;b'')';
  semicolon_ddl := 'CREATE INDEX dynamic_posts_status_idx ON posts(status)';
  EXECUTE semicolon_ddl;
  EXECUTE 'ALTER TABLE posts ADD CONSTRAINT posts_author_fk FOREIGN KEY (author_id) REFERENCES users(id)';
END
$$;

DO LANGUAGE "plpgsql" $quoted$
DECLARE
  conditional_ddl text := 'ALTER TABLE initial_comments ADD COLUMN ignored text';
BEGIN
  IF true THEN
    conditional_ddl := 'ALTER TABLE comments ADD COLUMN visible text';
  END IF;
  EXECUTE conditional_ddl;
END
$quoted$;

DO $$
BEGIN
  -- This is executable static SQL, handled by the base parser rather than dynamic extraction.
  ALTER TABLE direct_posts ADD COLUMN direct_status text;
END
$$;

CREATE OR REPLACE FUNCTION cleanup() RETURNS void LANGUAGE plpgsql AS $body$
BEGIN
  EXECUTE 'DROP INDEX obsolete';
  RAISE NOTICE 'EXECUTE ''CREATE TABLE ignored (id uuid)''';
END
$body$;

CREATE OR REPLACE FUNCTION default_argument(label text DEFAULT 'run AS admin')
RETURNS void LANGUAGE plpgsql AS $default_arg$
BEGIN
  EXECUTE 'ALTER TABLE accounts ADD COLUMN generated text';
END
$default_arg$;

CREATE OR REPLACE FUNCTION language_after_body() RETURNS void AS $later$
BEGIN
  EXECUTE format('DROP INDEX %I', index_name);
END
$later$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION sql_language() RETURNS void LANGUAGE sql AS $sql$
  SELECT 'EXECUTE ''CREATE TABLE ignored_sql_language (id uuid)''';
$sql$;

CREATE OR REPLACE FUNCTION direct_function_ddl() RETURNS void LANGUAGE plpgsql AS $function$
BEGIN
  ALTER TABLE function_posts ADD COLUMN direct_status text;
END
$function$;

CREATE OR REPLACE PROCEDURE direct_procedure_ddl() LANGUAGE plpgsql AS $procedure$
BEGIN
  ALTER TABLE procedure_posts ADD COLUMN direct_status text;
END
$procedure$;
