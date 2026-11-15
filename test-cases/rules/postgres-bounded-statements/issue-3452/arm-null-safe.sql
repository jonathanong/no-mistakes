DELETE FROM marks
WHERE owner = $1
  AND (state IS NOT DISTINCT FROM 'x' OR state = 'y');
