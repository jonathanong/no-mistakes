DO $tag$
BEGIN
  CREATE TABLE before_label (id integer);
  <<l>>--gap
  FOR i IN 1..1 LOOP
    RAISE NOTICE 'inside loop';
  END LOOP;
  CREATE TABLE after_label (id integer);
END;
$tag$;
