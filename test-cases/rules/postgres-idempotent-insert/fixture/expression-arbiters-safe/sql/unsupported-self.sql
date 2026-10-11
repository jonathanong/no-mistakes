-- An unsupported key is safe when the assigned column is provably unchanged.
INSERT INTO users (email, name) VALUES ('a', 'n')
ON CONFLICT ((CASE WHEN email IS NULL THEN name ELSE email END)) DO UPDATE SET name = name;
