-- Bare id is outer unless a declared UNNEST output exposes it.
DELETE FROM accounts WHERE id IN (SELECT id FROM unnest($1::uuid[]) LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT id FROM unnest($1::uuid[]) c LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT id FROM unnest($1::uuid[]) WITH ORDINALITY c(n) LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT id FROM unnest($1::uuid[], $2::uuid[]) c LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT id FROM unnest($1::uuid[]) id LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT id FROM unnest($1::uuid[]) c(id) LIMIT 1);
