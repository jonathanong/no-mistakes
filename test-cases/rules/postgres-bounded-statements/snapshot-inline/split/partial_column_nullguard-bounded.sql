-- Equality itself proves the predicate; NULL-safe equality does not.
SELECT 1 FROM partial_logins AS p WHERE p.email = $1;
