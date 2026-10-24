-- The unknown statement still reports even when END IF follows it.
DO $$
BEGIN
  IF true THEN
    CREATE TABLE audit_log (id bigint PRIMARY KEY);
    FROBULATE accounts;
  END IF;
END
$$;

DO $$
BEGIN
  IF true THEN
    FROBULATE first_after_if;
  END IF;
END
$$;

DO $$
BEGIN
  LOOP
    FROBULATE first_after_loop;
  END LOOP;
END
$$;
