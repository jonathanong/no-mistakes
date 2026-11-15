DELETE FROM codes_literal
WHERE owner = $1
  AND code = 'a'::text;
