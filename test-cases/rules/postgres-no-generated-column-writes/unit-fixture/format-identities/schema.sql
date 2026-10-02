CREATE TABLE dynamic_identifier (id int, computed int GENERATED ALWAYS AS (id + 1) STORED);
CREATE TABLE dynamic_value (id int, computed int GENERATED ALWAYS AS (id + 1) STORED);

DO $$
DECLARE
  dynamic_ddl text;
  copied_ddl text;
BEGIN
  -- Unknown identifiers and values stay broad facts, not concrete relation events.
  EXECUTE format('CREATE TABLE %I (id int, formatted_create int GENERATED ALWAYS AS (id + 1) STORED)', target_name);
  dynamic_ddl := format('ALTER TABLE %s ADD COLUMN formatted_alter int GENERATED ALWAYS AS (id + 1) STORED', target_name);
  EXECUTE dynamic_ddl;
  copied_ddl := dynamic_ddl;
  EXECUTE copied_ddl;

  EXECUTE format('DROP TABLE %I', target_name);
  dynamic_ddl := format('DROP TABLE %I', target_name);
  EXECUTE dynamic_ddl;
  copied_ddl := dynamic_ddl;
  EXECUTE copied_ddl;
  EXECUTE format('DROP TABLE %s', target_name);

  -- Literal values, escaped percent signs, and no-placeholder format calls
  -- keep a concrete relation and column history.
  EXECUTE format('CREATE TABLE literal_format (id int, escaped_percent int GENERATED ALWAYS AS (id + length(''%%'')) STORED, literal_value int GENERATED ALWAYS AS (id + length(%L)) STORED)', 'x');
  EXECUTE format('CREATE TABLE plain_format (id int, generated int GENERATED ALWAYS AS (id + 1) STORED)');
END
$$;
