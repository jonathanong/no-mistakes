DELETE FROM marks
WHERE owner = $1
  AND (state = $2 OR state = $3);
