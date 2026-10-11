-- One invalid statement must not prevent static statements from retaining facts.
SELECT id FROM users WHERE id = 1;
SET 42 = 'invalid';
SET app. 42 = 'invalid';
SET;
SELECT set_config('statement_timeout', '1s', true);
