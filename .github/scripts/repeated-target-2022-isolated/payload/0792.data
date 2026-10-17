-- Binary, cast, unary, and constants all preserve their column dependencies.
INSERT INTO users (email) VALUES ('a')
ON CONFLICT ((lower(email || '.com'))) DO UPDATE SET email = 'b';
INSERT INTO users (email) VALUES ('a')
ON CONFLICT ((lower(email::text))) DO UPDATE SET email = 'b';
INSERT INTO users (score) VALUES (1)
ON CONFLICT ((abs(-score))) DO UPDATE SET score = 2;
-- An operator class changes index comparison behavior; do not claim full proof.
INSERT INTO users (email, name) VALUES ('a', 'n')
ON CONFLICT (lower(email) text_pattern_ops) DO UPDATE SET name = 'b';
