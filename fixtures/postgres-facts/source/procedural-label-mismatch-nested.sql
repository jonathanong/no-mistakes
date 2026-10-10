DO $$ BEGIN IF true THEN <<x>> LOOP RAISE NOTICE 'x'; END LOOP y; CREATE TABLE later_sql (id int); END IF; END $$;
