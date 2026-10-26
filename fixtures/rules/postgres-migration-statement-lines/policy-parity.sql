-- Schema and statement-only policies share the recovered lexical statement heads.
DO $$ BEGIN
  CREATE
    TABLE policy_first (id uuid);
  ALTER
    TABLE policy_first ADD COLUMN value text;
  CREATE
    TABLE policy_second (id uuid);
  ALTER
    TABLE policy_second ADD COLUMN value text;
END $$;
CREATE FUNCTION policy_schema() RETURNS void LANGUAGE plpgsql AS $$
BEGIN
  CREATE
    TABLE routine_first (id uuid);
  ALTER
    TABLE routine_first ADD COLUMN value text;
  CREATE
    TABLE routine_second (id uuid);
  ALTER
    TABLE routine_second ADD COLUMN value text;
END
$$;
