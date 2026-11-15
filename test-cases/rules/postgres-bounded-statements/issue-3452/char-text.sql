DELETE FROM codes
WHERE owner = $1
  AND code = 'a ';
