-- Routine definitions expose broad policy metadata but execute no table changes.
CREATE FUNCTION dormant_only() RETURNS void LANGUAGE plpgsql AS $$
BEGIN
  CREATE TABLE ghost (id int, computed int GENERATED ALWAYS AS (id + 1) STORED);
  ALTER TABLE ghost ADD COLUMN second_computed int GENERATED ALWAYS AS (id + 2) STORED;
END;
$$;
