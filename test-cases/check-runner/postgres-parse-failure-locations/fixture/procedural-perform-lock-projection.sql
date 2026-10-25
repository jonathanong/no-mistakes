-- The compatibility rewrite must preserve the original function occurrence span.
PERFORM abs(id) FROM accounts WHERE id = 1 FOR KEY SHARE;
