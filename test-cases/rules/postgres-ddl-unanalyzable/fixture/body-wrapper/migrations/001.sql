-- A body child never lends its advisory classification to a broken header.
CREATE FUNCTION malformed_wrapper() RETURNS void LANGUAGE plpgsql BOGUS AS $$
BEGIN
  FROBULATE child_of_malformed_wrapper;
END
$$;
ALTER TABLE malformed_tail ADD COLUMN;
