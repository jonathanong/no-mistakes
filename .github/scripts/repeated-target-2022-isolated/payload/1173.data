-- CASE dependencies require a broader proof than this rule currently supports.
INSERT INTO users (email, name) VALUES ('a', 'n')
ON CONFLICT ((CASE WHEN email IS NULL THEN name ELSE email END)) DO UPDATE SET name = 'b';
