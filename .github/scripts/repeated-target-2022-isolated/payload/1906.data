-- Non-recursive self-names read the physical relation before their output binds.
CREATE VIEW app.summary ("Total") AS
WITH accounts AS (SELECT id FROM accounts), recent AS (SELECT id FROM accounts)
SELECT count(*) FROM recent JOIN app.orders o ON o.account_id = recent.id;
CREATE MATERIALIZED VIEW "App"."Rollup" AS SELECT * FROM "App"."Orders";
CREATE TEMP VIEW scratch AS SELECT 1;
CREATE VIEW nested_scope AS WITH rows AS (SELECT * FROM base_rows)
SELECT * FROM rows WHERE EXISTS (WITH rows AS (SELECT * FROM rows) SELECT 1 FROM rows JOIN more_rows ON true);
CREATE VIEW recursive_scope AS WITH RECURSIVE chain AS (SELECT id FROM roots UNION ALL SELECT r.id FROM chain c JOIN branches r ON r.parent = c.id) SELECT * FROM chain;
CREATE OR REPLACE FUNCTION app.adjust(IN amount numeric DEFAULT 1, INOUT label text DEFAULT 'x') RETURNS text LANGUAGE sql IMMUTABLE STRICT PARALLEL SAFE SECURITY INVOKER SET search_path TO app AS $$ SELECT label $$;
CREATE FUNCTION app.values_for(id uuid) RETURNS SETOF app."Row" LANGUAGE sql STABLE AS $$ SELECT * FROM app.rows WHERE owner_id = id $$;
CREATE FUNCTION app.audit() RETURNS trigger LANGUAGE plpgsql VOLATILE SECURITY DEFINER AS $$ BEGIN RETURN NEW; END $$;
CREATE OR REPLACE TRIGGER audit_rows AFTER INSERT ON app.rows REFERENCING NEW TABLE AS new_rows FOR EACH STATEMENT EXECUTE FUNCTION app.audit();
CREATE CONSTRAINT TRIGGER row_guard AFTER UPDATE OF id ON app.rows FROM app.parents DEFERRABLE INITIALLY DEFERRED FOR EACH ROW WHEN (NEW.id IS DISTINCT FROM OLD.id) EXECUTE PROCEDURE app.audit();
-- Table functions are not physical relations; nested queries still contribute reads.
CREATE VIEW function_rows AS SELECT * FROM generate_series(1, 2);
CREATE VIEW repeated_rows AS SELECT a.id FROM app.orders a JOIN app.orders b ON a.id = b.id;
CREATE VIEW forward_ctes AS WITH RECURSIVE first_rows AS (SELECT * FROM later_rows), later_rows AS (SELECT * FROM physical_rows) SELECT * FROM first_rows;
CREATE FUNCTION app.output_only(OUT total integer) LANGUAGE sql AS $$ SELECT 1 $$;
CREATE TRIGGER argument_rows BEFORE INSERT OR DELETE ON app.rows FOR EACH ROW EXECUTE FUNCTION app.audit('it''s', 42, NULL, integer, INTEGER, MiXeD, "MiXeD");
