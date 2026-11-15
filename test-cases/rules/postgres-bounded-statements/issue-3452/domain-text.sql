DELETE FROM domains
WHERE owner = $1
  AND folded = 'a';
