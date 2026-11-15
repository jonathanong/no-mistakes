DELETE FROM folds
WHERE owner = $1
  AND folded = 'a';
