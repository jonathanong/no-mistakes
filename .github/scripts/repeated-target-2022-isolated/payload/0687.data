DO $$ BEGIN
  CREATE TABLE before_batch (id int);
  <<first>> FOR i IN 1..0 LOOP RAISE NOTICE 'first'; END LOOP;
  <<second>> FOR i IN 1..0 LOOP RAISE NOTICE 'second'; END LOOP;
  <<third>> FOR i IN 1..0 LOOP RAISE NOTICE 'third'; END LOOP;
  <<fourth>> FOR i IN 1..0 LOOP RAISE NOTICE 'fourth'; END LOOP;
  <<fifth>> FOR i IN 1..0 LOOP RAISE NOTICE 'fifth'; END LOOP;
  CREATE TABLE after_batch (id int);
END $$;
