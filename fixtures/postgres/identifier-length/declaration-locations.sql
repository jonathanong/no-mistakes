-- Routine body DDL must not consume the later outer table's location.
CREATE FUNCTION body_owner() RETURNS void LANGUAGE plpgsql AS $$
BEGIN
  CREATE TABLE repeated_table (id int);
END
$$;
CREATE TABLE repeated_table (id int); -- no-mistakes-disable-line postgres-identifier-length

/* outer /* CREATE TABLE decoy_table (id int); */ still a comment */
CREATE TEMP TABLE temporary_table (id int);
CREATE UNLOGGED TABLE unlogged_table (id int);
CREATE TABLE invalid_table AS SELECT FROM;
CREATE TABLE after_invalid (id int);
CREATE OR REPLACE VIEW replaced_view (explicit_column) AS SELECT 1;
CREATE TEMP VIEW temporary_view AS SELECT 1;
CREATE OR REPLACE TRIGGER replaced_trigger BEFORE INSERT ON repeated_table
FOR EACH ROW EXECUTE FUNCTION touch_row();
CREATE CONSTRAINT TRIGGER constraint_trigger AFTER INSERT ON repeated_table
DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION touch_row();
