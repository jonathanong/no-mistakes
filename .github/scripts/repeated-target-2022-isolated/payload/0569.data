EXPLAIN CREATE TABLE wrapper_children (
  id integer,
  CONSTRAINT wrapper_check CHECK (id > 0)
);
DO $body$
BEGIN
  EXECUTE 'CREATE TABLE decoded_children (id integer, CONSTRAINT decoded_check CHECK (id > 0));';
END
$body$;
