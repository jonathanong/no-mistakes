-- Comments between declaration tokens do not change the identifier.
CREATE /* declaration */ PROCEDURE /* name */ commented_procedure() LANGUAGE plpgsql AS $$BEGIN NULL; END$$;
CREATE PROCEDURE app.U&"escaped_\0070rocedure"() LANGUAGE plpgsql AS $$BEGIN NULL; END$$;
CREATE PROCEDURE U&"custom_!0070rocedure" UESCAPE '!'() LANGUAGE plpgsql AS $$BEGIN NULL; END$$;
DO $$
BEGIN
  EXECUTE 'CREATE PROCEDURE dynamic_procedure() LANGUAGE plpgsql AS $body$BEGIN NULL; END$body$';
END
$$;
