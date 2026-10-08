-- Argument order is deliberate: consumers need facts, not a replay policy.
INSERT INTO public."café" AS target (id, value) VALUES (1, 2)
ON CONFLICT (id) DO UPDATE SET
  value = COALESCE(target.value, CURRENT_TIMESTAMP),
  reversed = COALESCE(CURRENT_TIMESTAMP, target.value),
  direct = EXCLUDED.value,
  nested = lower(COALESCE(EXCLUDED.value, target.value)) || upper('λ'),
  choice = CASE target.id WHEN 1 THEN EXCLUDED.value WHEN 2 THEN target.value ELSE 'λ' END,
  searched = CASE WHEN target.id > 0 THEN COALESCE(target.value, EXCLUDED.value) END,
  wrapped = -(CAST((COALESCE(target.id, EXCLUDED.id)) AS integer)),
  opaque = sum(DISTINCT target.id) FILTER (WHERE target.id > 0),
  wildcard = count(*),
  windowed = sum(target.id) OVER (),
  named = custom_call(first_arg => target.value, second_arg => EXCLUDED.value),
  null_treatment = first_value(target.id) IGNORE NULLS,
  direct_target = target.value;
