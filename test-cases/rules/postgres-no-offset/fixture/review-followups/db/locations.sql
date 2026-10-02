-- OFFSET 90 in a comment is not a clause.
SELECT 'OFFSET 80', "雪"
FROM t
OFFSET
  1;
/* outer /* nested ; */ still comment ; */
SELECT E'escaped\'quote; OFFSET 50';
SELECT 1
OFFSET 0;
