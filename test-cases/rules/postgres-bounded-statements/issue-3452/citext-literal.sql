DELETE FROM folds_literal
WHERE owner = $1
  AND folded = 'a'::text;
