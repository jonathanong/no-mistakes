DO $body$
BEGIN
  IF ready THEN
    EXECUTE $sql$INSERT INTO prompts (body) VALUES ('if')$sql$;
    INSERT INTO prompts (body) VALUES ('neighbor');
  ELSIF fallback THEN
    EXECUTE 'INSERT INTO prompts (body) VALUES (''elsif'')';
  ELSE
    IF nested THEN
      EXECUTE query_text;
      EXECUTE 'INSERT INTO prompts (body) VALUES (''nested'')';
    END IF;
  END IF;
  -- EXECUTE after CASE THEN is a column, never a procedural command.
  SELECT CASE WHEN enabled THEN execute ELSE 'none' END FROM flags;
END
$body$;
CREATE TABLE after_conditional_execute (id integer);
