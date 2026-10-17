-- Trivia must not make a zero-argument call's span end at its name.
INSERT INTO items (id, value) VALUES (1, 2)
ON CONFLICT (id) DO UPDATE SET value = COALESCE(items.value, now /*keep*/ ());
INSERT INTO items (id, value) SELECT 2, now /*keep*/ ();
-- An unproven first operand must not erase a proven later sibling's location.
INSERT INTO items (id, value) VALUES (3, 4)
ON CONFLICT (id) DO UPDATE SET value = COALESCE(-(items.value), EXCLUDED.value);
-- An expression argument name is accepted syntax but has no identifier contract.
INSERT INTO items AS target (id, value) VALUES (5, 6)
ON CONFLICT (id) DO UPDATE SET value = COALESCE(custom_call((1 + 2) => EXCLUDED.value), target.value);
