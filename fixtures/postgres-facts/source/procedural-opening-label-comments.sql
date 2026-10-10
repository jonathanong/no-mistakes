DO $$ BEGIN
  CREATE TABLE before_label (id int);
  <<retry>> /* nested /* comment */ gap */ FOR i IN 1..0 LOOP
    RAISE NOTICE 'ignored';
  END LOOP;
  <<again>> -- line comment between label and loop
  FOR i IN 1..0 LOOP
    RAISE NOTICE 'ignored too';
  END LOOP;
  CREATE TABLE after_label (id int);
END $$;
