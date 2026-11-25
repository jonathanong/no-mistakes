BEGIN;
RELEASE SAVEPOINT missing_release;
ROLLBACK TO SAVEPOINT missing_rollback;
ROLLBACK;

SET default_transaction_read_only = DEFAULT;
SELECT set_config(current_setting('app.key'), 'on', false);
SELECT set_config('default_transaction_read_only', current_setting('app.mode'), false);
SELECT set_config('default_transaction_read_only', 'on', current_setting('app.local')::boolean);
SELECT set_config('default_transaction_read_only', 'on');
SELECT set_config(1, 'on', false);
SELECT set_config('default_transaction_read_only', 123, false);
SELECT set_config(setting_name => 'default_transaction_read_only', new_value => 'on', is_local => false);
SELECT set_config('work_mem', '64MB', false);

EXPLAIN (ANALYZE $1) UPDATE sample_jobs SET status = 'ready' WHERE id = 47;
EXPLAIN (CUSTOM_OPT TRUE) DELETE FROM sample_jobs WHERE id = 48;
EXPLAIN (ANALYZE TRUE, GENERIC_PLAN TRUE) INSERT INTO sample_jobs (id) VALUES (49);
EXPLAIN (CUSTOM_OPT TRUE) SELECT id FROM sample_jobs;
