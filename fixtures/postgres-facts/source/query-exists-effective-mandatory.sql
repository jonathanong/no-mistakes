-- Direct negative-existence guard remains required despite syntactic mandatory=false.
INSERT INTO t (id) SELECT 1 WHERE NOT EXISTS (SELECT 1);
-- NOT distributes over AND, so the EXISTS arm is optional.
INSERT INTO t (id) SELECT 1 WHERE NOT (false AND EXISTS (SELECT 1));
-- NOT distributes over OR, so the EXISTS arm is required.
INSERT INTO t (id) SELECT 1 WHERE NOT (true OR EXISTS (SELECT 1));
INSERT INTO t (id) SELECT 1 WHERE NOT (NOT EXISTS (SELECT 1));
INSERT INTO t (id) SELECT 1 WHERE NOT (NOT (NOT EXISTS (SELECT 1)));
-- The inner OR is conjunctive under NOT, but its outer AND is disjunctive.
INSERT INTO t (id) SELECT 1 WHERE NOT (false AND (true OR EXISTS (SELECT 1)));
-- Both enclosing ORs are conjunctive under NOT.
INSERT INTO t (id) SELECT 1 WHERE NOT (true OR (false OR EXISTS (SELECT 1)));
-- An un-negated OR is optional even when the EXISTS arm itself is negative.
INSERT INTO t (id) SELECT 1 WHERE true OR NOT EXISTS (SELECT 1);
-- The projection occurrence is not a WHERE predicate.
SELECT NOT EXISTS (SELECT 1);
-- The two set branches retain distinct occurrence and scope facts.
INSERT INTO t (id) SELECT 1 WHERE NOT EXISTS (SELECT 1)
UNION ALL SELECT 2 WHERE NOT (false AND EXISTS (SELECT 1));
-- Opaque boolean wrappers cannot provide a conjunctiveness proof.
SELECT 1 WHERE EXISTS (SELECT 1) IS FALSE;
SELECT 1 WHERE NOT CASE WHEN EXISTS (SELECT 1) THEN true ELSE false END;
