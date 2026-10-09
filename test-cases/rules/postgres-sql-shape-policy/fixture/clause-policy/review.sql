-- Arbiter predicates belong to WHERE for both actions; arbiter expressions do not.
INSERT INTO events(id) VALUES (1) ON CONFLICT (id) WHERE uuidv7() IS NOT NULL DO NOTHING;
INSERT INTO events(id) VALUES (2) ON CONFLICT (id) WHERE uuidv7() IS NOT NULL DO UPDATE SET id = 2;
INSERT INTO events(id) VALUES (3) ON CONFLICT ((uuidv7())) DO NOTHING;
SELECT 1 WHERE "custom.schema"."odd.function"() IS NOT NULL;
SELECT 1 WHERE "Upper"."quote""function"() IS NOT NULL;
SELECT 1 WHERE _simple$9() IS NOT NULL;
SELECT 1 WHERE "select"() IS NOT NULL;
