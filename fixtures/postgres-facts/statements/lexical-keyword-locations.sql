-- INSERT INTO ignored SELECT
/* outer SELECT
   /* nested INSERT INTO ignored */ still SELECT */
SELECT 'SELECT ''quoted''
INSERT INTO ignored' AS text,
       "SELECT
INSERT INTO ignored" AS name,
       $tag$SELECT
INSERT INTO ignored$tag$ AS body,
       $$SELECT
INSERT INTO ignored$$ AS plain,
       foo$tag$ AS identifier;
  INSERT /* separator */ INTO real_table VALUES (1);
    INSERT INTO real_table VALUES (2);
-- SELECT hidden
  SELECT 42;
