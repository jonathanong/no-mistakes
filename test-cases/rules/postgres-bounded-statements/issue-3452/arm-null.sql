DELETE FROM marks
WHERE owner = $1
  AND (state = 'x' OR state IS NULL);
