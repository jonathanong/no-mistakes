-- A boolean primary key used directly is an implicit comparison to true.
SELECT 1 FROM boolean_keys WHERE enabled;
SELECT 1 FROM boolean_keys b WHERE b.enabled;
SELECT 1 FROM boolean_keys WHERE enabled = true;
SELECT 1 FROM boolean_keys b WHERE b.enabled = true;
SELECT 1 FROM boolean_keys WHERE enabled AND archived;

-- A boolean column outside the key cannot bound the table.
SELECT 1 FROM boolean_keys WHERE archived;
SELECT 1 FROM boolean_keys b WHERE b.archived;

-- Each column of a composite key must be pinned in the same conjunction.
SELECT 1 FROM composite_boolean_keys WHERE tenant_id = $1 AND enabled;
SELECT 1 FROM composite_boolean_keys c WHERE c.enabled AND c.tenant_id = $1;
SELECT 1 FROM composite_boolean_keys WHERE enabled;
SELECT 1 FROM composite_boolean_keys WHERE tenant_id = $1;
SELECT 1 FROM composite_boolean_keys WHERE tenant_id = $1 AND archived;

-- Alternatives and negation do not acquire an implicit true pin.
SELECT 1 FROM composite_boolean_keys WHERE tenant_id = $1 AND (enabled OR archived);
SELECT 1 FROM composite_boolean_keys WHERE tenant_id = $1 AND NOT enabled;
SELECT 1 FROM boolean_keys WHERE enabled OR archived;
SELECT 1 FROM boolean_keys WHERE NOT enabled;
