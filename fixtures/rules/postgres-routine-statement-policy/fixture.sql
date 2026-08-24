DO $$
BEGIN
  CREATE TABLE do_table (id uuid);
  ALTER TABLE do_table ADD CONSTRAINT do_check CHECK (true);
  CREATE INDEX do_table_id_idx ON do_table(id);
  CREATE VIEW do_view AS SELECT id FROM do_table;
  TRUNCATE do_table;
  DROP INDEX do_table_id_idx;
  DROP VIEW do_view;
END
$$;

CREATE FUNCTION routine_policy_function() RETURNS void LANGUAGE plpgsql AS $function$
BEGIN
  CREATE TABLE function_table (id uuid);
  ALTER TABLE function_table ADD CONSTRAINT function_check CHECK (true);
  CREATE INDEX function_table_id_idx ON function_table(id);
  CREATE VIEW function_view AS SELECT id FROM function_table;
  TRUNCATE function_table;
  DROP INDEX function_table_id_idx;
  DROP VIEW function_view;
END
$function$;

CREATE PROCEDURE routine_policy_procedure() LANGUAGE plpgsql AS $procedure$
BEGIN
  CREATE TABLE procedure_table (id uuid);
  ALTER TABLE procedure_table ADD CONSTRAINT procedure_check CHECK (true);
  CREATE INDEX procedure_table_id_idx ON procedure_table(id);
  CREATE VIEW procedure_view AS SELECT id FROM procedure_table;
  TRUNCATE procedure_table;
  DROP INDEX procedure_table_id_idx;
  DROP VIEW procedure_view;
END
$procedure$;
