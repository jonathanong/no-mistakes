CREATE TABLE before_execute (id integer);
DO $body$
BEGIN
  -- Semicolons inside literal SQL do not delimit the procedural wrapper.
  EXECUTE $sql$/* nested */ INSERT INTO prompts (body) VALUES ('hi'); INSERT INTO prompts (body) VALUES ('snow''s');$sql$;
  EXECUTE 'INSERT INTO prompts (body) VALUES (''quoted''''s'')';
  EXECUTE 'INSERT INTO prompts (body) VALUES (';
  EXECUTE query_text;
  EXECUTE 'INSERT INTO prompts (body) VALUES (1)' || suffix;
  EXECUTE format('INSERT INTO %I VALUES (1)', table_name);
END
$body$;
CREATE TABLE after_execute (id integer);
