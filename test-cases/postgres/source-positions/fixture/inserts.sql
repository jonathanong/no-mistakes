-- Quoted INSERT INTO and OVERRIDING must not affect real source ranges.
SELECT 'INSERT INTO decoy OVERRIDING USER VALUE';
INSERT INTO items (id) VALUES (1);
INSERT /* comment */ INTO items (id) OVERRIDING USER VALUE VALUES (2);
INSERT INTO items (id) VALUES (3);
