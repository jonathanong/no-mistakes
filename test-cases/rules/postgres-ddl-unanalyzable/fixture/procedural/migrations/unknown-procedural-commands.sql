-- Procedural command prefixes do not hide malformed SQL payloads.
DO $$
BEGIN
  PERFORM 1 + );
END
$$;

DO $$
BEGIN
  OPEN missing_cursor FOR FROBULATE accounts;
END
$$;

DO $$
DECLARE bad_cursor CURSOR FOR FROBULATE accounts;
BEGIN
  NULL;
END
$$;
