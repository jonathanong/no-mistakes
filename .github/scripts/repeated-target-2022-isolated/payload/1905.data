-- Keep this fixture syntactic: typed temporal text such as 'now' is not a volatility policy.
INSERT INTO public.events (id, value) VALUES (1, 2)
ON CONFLICT (id) DO UPDATE SET
  null_check = target."nullable" IS NULL,
  not_null_check = EXCLUDED."nullable" IS NOT NULL,
  distinct_check = target."nullable" IS DISTINCT FROM EXCLUDED."nullable",
  not_distinct_check = target."nullable" IS NOT DISTINCT FROM EXCLUDED."nullable",
  and_check = target."nullable" IS NULL AND EXCLUDED."nullable" IS NOT NULL,
  or_check = (target."nullable" IS NULL OR EXCLUDED."nullable" IS NOT NULL),
  not_check = NOT (target."nullable" IS NULL),
  case_check = CASE WHEN target."nullable" IS NULL THEN custom_call($2) ELSE $1::text END,
  parameter_value = custom_call(COALESCE($3::uuid, $4)),
  -- NULL and 'NULL' are different literal values; retain both the kind and spelling.
  null_literal = NULL,
  string_null_literal = 'NULL',
  boolean_literal = TRUE,
  -- Keep this beyond JavaScript's safe integer range to pin decimal precision.
  number_literal = 900719925474099312345,
  escaped_string_literal = 'it''s',
  literal_variants = custom_call(E'it\'s', $$dollar 'quoted'$$, X'AB', U&'snowman', N'café'),
  nested_literals = custom_call(NULL, 'NULL', TRUE, 900719925474099312345, 'it''s'),
  case_literals = CASE WHEN TRUE THEN NULL ELSE 'NULL' END,
  timestamp_value = TIMESTAMP 'now',
  timestamp_epoch_value = TIMESTAMP 'epoch',
  date_value = DATE 'today',
  timestamp_local_value = TIMESTAMP WITHOUT TIME ZONE '2025-01-02 03:04:05',
  timestamp_tz_value = TIMESTAMP WITH TIME ZONE '2025-01-02 03:04:05+00',
  time_local_value = TIME WITHOUT TIME ZONE '03:04:05',
  time_tz_value = TIME WITH TIME ZONE '03:04:05+00';
