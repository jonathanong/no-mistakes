DO $$ BEGIN
  CREATE TABLE before_unlabeled (id int);
  IF TRUE THEN
    FOR i IN 1..0 LOOP RAISE NOTICE 'tick'; END LOOP;
  END IF;
  CREATE TABLE after_unlabeled (id int);
END $$;
