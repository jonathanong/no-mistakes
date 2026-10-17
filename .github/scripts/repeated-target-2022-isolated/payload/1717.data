-- A peeled procedural body keeps lock facts even without an outer FOR token.
DO $$
  SELECT id FROM users WHERE id IN (1, 2) FOR UPDATE;
$$;
