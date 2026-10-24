-- Malformed headers do not become valid merely because their bodies are parsed.
CREATE FUNCTION malformed_function() RETURNS void LANGUAGE plpgsql BOGUS AS $$
BEGIN
  PERFORM 1;
END
$$;

CREATE PROCEDURE malformed_procedure() LANGUAGE plpgsql BOGUS AS $$
BEGIN
  PERFORM 2;
END
$$;
