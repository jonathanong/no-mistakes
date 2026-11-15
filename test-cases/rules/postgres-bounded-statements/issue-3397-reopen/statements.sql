-- pass verbatim-table
WITH candidates AS (
  SELECT current_flow.id,
    CASE
      WHEN current_flow.provider = 'x'
        AND current_flow.callback_received_at <= CURRENT_TIMESTAMP - make_interval(secs => $4)
        THEN 'expired'
      WHEN current_flow.status = 'exchanging' THEN 'unknown'
      WHEN current_flow.exchange_attempts >= $3 THEN 'failed'
      ELSE 'unknown'
    END AS callback_error
  FROM flows flow
  JOIN flows current_flow ON current_flow.id = flow.id
  WHERE flow.status = 'callback_received'
  ORDER BY current_flow.id
  LIMIT $1
)
UPDATE flows
SET callback_error = candidates.callback_error
FROM candidates
WHERE flows.id = candidates.id;

-- pass view-nested
WITH candidates AS (
  SELECT current_flow.id,
    CASE
      WHEN current_flow.provider = 'x'
        AND current_flow.callback_received_at <= CURRENT_TIMESTAMP - make_interval(secs => $4)
        THEN 'expired'
      WHEN current_flow.status = 'exchanging' THEN 'unknown'
      WHEN current_flow.exchange_attempts >= $3 THEN 'failed'
      ELSE 'unknown'
    END AS callback_error
  FROM flows flow
  JOIN view_current_flows current_flow ON current_flow.id = flow.id
  WHERE flow.status = 'callback_received'
  ORDER BY current_flow.id
  LIMIT $1
),
terminalized AS (
  UPDATE flows
  SET callback_error = candidates.callback_error
  FROM candidates
  WHERE flows.id = candidates.id
    AND flows.status = 'callback_received'
  RETURNING id
)
SELECT id FROM terminalized;

-- pass unkeyed-view
WITH candidates AS (
  SELECT current_flow.id,
    CASE
      WHEN current_flow.provider = 'x'
        AND current_flow.callback_received_at <= CURRENT_TIMESTAMP - make_interval(secs => $4)
        THEN 'expired'
      WHEN current_flow.status = 'exchanging' THEN 'unknown'
      WHEN current_flow.exchange_attempts >= $3 THEN 'failed'
      ELSE 'unknown'
    END AS callback_error
  FROM flows flow
  JOIN view_flow_pairs current_flow ON current_flow.id = flow.id
  WHERE flow.status = 'callback_received'
  ORDER BY current_flow.id
  LIMIT $1
)
UPDATE flows
SET callback_error = candidates.callback_error
FROM candidates
WHERE flows.id = candidates.id;

-- pass assignment-case
WITH candidates AS (
  SELECT current_flow.id,
    CASE
      WHEN current_flow.provider = 'x'
        AND current_flow.callback_received_at <= CURRENT_TIMESTAMP - make_interval(secs := $4)
        THEN 'expired'
      WHEN current_flow.status = 'exchanging' THEN 'unknown'
      WHEN current_flow.exchange_attempts >= $3 THEN 'failed'
      ELSE 'unknown'
    END AS callback_error
  FROM flows flow
  JOIN flows current_flow ON current_flow.id = flow.id
  WHERE flow.status = 'callback_received'
  ORDER BY current_flow.id
  LIMIT $1
)
UPDATE flows
SET callback_error = candidates.callback_error
FROM candidates
WHERE flows.id = candidates.id;

-- pass bare-current-timestamp
SELECT CURRENT_TIMESTAMP FROM flows LIMIT 1;

-- report no-limit
WITH candidates AS (
  SELECT current_flow.id
  FROM flows flow
  JOIN flows current_flow ON current_flow.id = flow.id
  WHERE flow.status = 'callback_received'
)
UPDATE flows
SET callback_error = NULL
FROM candidates
WHERE flows.id = candidates.id;

-- report nonunique-join
WITH candidates AS (
  SELECT current_flow.id
  FROM flows flow
  JOIN flows current_flow ON current_flow.status = flow.status
  WHERE flow.status = 'callback_received'
  LIMIT $1
)
UPDATE flows
SET callback_error = NULL
FROM candidates
WHERE flows.status = candidates.id;

-- report unknown-call
SELECT not_a_builtin(id) FROM flows LIMIT 1;

-- report unknown-case
SELECT CASE WHEN true THEN not_a_builtin($1) END FROM flows LIMIT 1;

-- report other-schema
SELECT app.current_timestamp() FROM flows LIMIT 1;

-- report unlimited-current-timestamp
SELECT CURRENT_TIMESTAMP FROM flows;
