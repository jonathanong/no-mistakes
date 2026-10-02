CREATE FUNCTION return_modes() RETURNS SETOF integer LANGUAGE plpgsql AS $$
DECLARE
  "RETURN" text;
BEGIN
  "RETURN" := 'still running';
  CREATE TABLE after_quoted_return_assignment (id int);
  RETURN NEXT 1;
  CREATE TABLE after_return_next (id int);
  RETURN QUERY SELECT 2;
  CREATE TABLE after_return_query (id int);
  IF true THEN
    RETURN;
  END IF;
  CREATE TABLE after_conditional_return (id int);
  RETURN;
  CREATE TABLE after_unconditional_return (id int);
END;
$$;
