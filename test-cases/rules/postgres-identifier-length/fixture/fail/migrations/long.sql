SELECT 1;

CREATE TABLE accounts (
  id uuid,
  column_namexxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx text,
  CONSTRAINT inline_constraintxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx CHECK (id IS NOT NULL),
  CONSTRAINT table_constraintxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx FOREIGN KEY (id) REFERENCES referenced_onlyxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx (id)
);

CREATE TABLE table_namexxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx (
  id uuid,
  yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy text
);

CREATE INDEX idx_invoice_line_items_account_id_created_at_status_currency_code ON accounts (id);
CREATE UNIQUE INDEX unique_indexxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx ON accounts (id);

CREATE TRIGGER trigger_namexxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
  BEFORE INSERT ON accounts
  FOR EACH ROW EXECUTE FUNCTION touch_row();

CREATE FUNCTION function_namexxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx() RETURNS void LANGUAGE plpgsql AS $function$
BEGIN
  SELECT 1;
END
$function$;

CREATE OR REPLACE FUNCTION replaced_functionxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx() RETURNS void LANGUAGE plpgsql AS $function$
BEGIN
  SELECT 1;
END
$function$;

CREATE PROCEDURE procedure_namexxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx() LANGUAGE plpgsql AS $procedure$
BEGIN
  SELECT 1;
END
$procedure$;

CREATE OR REPLACE PROCEDURE replaced_procedurexxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx() LANGUAGE plpgsql AS $procedure$
BEGIN
  SELECT 1;
END
$procedure$;

CREATE VIEW view_namexxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx AS SELECT id FROM accounts;
CREATE MATERIALIZED VIEW matview_namexxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx AS SELECT id FROM accounts;
CREATE TYPE enum_typexxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx AS ENUM ('ready');

ALTER TABLE accounts ADD CONSTRAINT added_constraintxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx CHECK (id IS NOT NULL);
ALTER TABLE accounts ADD COLUMN added_columnxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx text;
ALTER TABLE accounts RENAME TO renamed_tablexxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
ALTER TABLE accounts RENAME COLUMN id TO renamed_columnxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
ALTER TABLE accounts RENAME CONSTRAINT added_constraintxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx TO renamed_constraintxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
CREATE INDEX short_idx ON accounts (id);
ALTER INDEX short_idx RENAME TO renamed_indexxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;
ALTER TABLE accounts DROP COLUMN id;

DO $$
BEGIN
  CREATE INDEX do_indexxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx ON accounts (id);
END $$;
