-- A literal NULL contributes no count when another supported argument supplies it.
SELECT 1 FROM accounts LIMIT COALESCE(NULL, 100);
SELECT 1 FROM accounts LIMIT COALESCE(100, NULL);
SELECT 1 FROM accounts LIMIT COALESCE(NULL, NULL, 100);
SELECT 1 FROM accounts LIMIT COALESCE((NULL), 100);
SELECT 1 FROM accounts LIMIT COALESCE(NULL, COALESCE(NULL, 100));
SELECT 1 FROM accounts LIMIT LEAST(NULL, 100);
SELECT 1 FROM accounts LIMIT GREATEST(100, NULL);
SELECT 1 FROM accounts LIMIT COALESCE((COALESCE(NULL, NULL)), '100')::integer;
SELECT 1 FROM accounts LIMIT COALESCE($1, NULL, 100);
SELECT 1 FROM accounts LIMIT COALESCE($1, $2);
-- All-NULL results and data-dependent or unsupported arguments remain conservative.
SELECT 1 FROM accounts LIMIT COALESCE(NULL, NULL);
SELECT 1 FROM accounts LIMIT COALESCE(NULL, get_limit());
SELECT 1 FROM accounts LIMIT COALESCE(100, get_limit());
SELECT 1 FROM accounts LIMIT app.coalesce(NULL, 100);
SELECT 1 FROM accounts LIMIT "coalesce"(NULL, 100);
SELECT 1 FROM accounts LIMIT COALESCE(NULL, accounts.id);
SELECT 1 FROM accounts LIMIT NULLIF(1, 1);
-- A caller-owned bind alone does not prove a non-NULL fallback.
SELECT 1 FROM accounts LIMIT COALESCE($1, NULL);
SELECT 1 FROM accounts LIMIT COALESCE(NULL, $1);
SELECT 1 FROM accounts LIMIT LEAST($1, NULL);
SELECT 1 FROM accounts LIMIT GREATEST(NULL, $1);
-- Parser-accepted unsupported argument forms must never establish a cap.
SELECT 1 FROM accounts LIMIT LEAST(*);
SELECT 1 FROM accounts LIMIT LEAST(value => 100);
