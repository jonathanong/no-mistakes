BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY;
SET LOCAL standard_conforming_strings = on;
SET LOCAL search_path = pg_catalog;
WITH relations AS (
  SELECT c.oid, c.relname, c.relkind FROM pg_class c
  JOIN pg_namespace n ON n.oid = c.relnamespace
  WHERE n.nspname = __SCHEMA__ AND c.relkind IN ('r', 'p')
), tables AS (
  SELECT quote_ident(r.relname) AS name, jsonb_build_object(
    'relationKind', CASE r.relkind WHEN 'p' THEN 'partitioned table' ELSE 'table' END,
    'columns', COALESCE((SELECT jsonb_object_agg(a.attname, jsonb_build_object(
      'dataType', format_type(a.atttypid, a.atttypmod), 'nullable', NOT a.attnotnull,
      'ordinalPosition', a.attnum,
      'defaultExpression', CASE WHEN a.attgenerated = '' THEN pg_get_expr(d.adbin, d.adrelid) END,
      'generated', CASE a.attgenerated WHEN 's' THEN 'stored' WHEN 'v' THEN 'virtual' END,
      'generatedExpression', CASE WHEN a.attgenerated <> '' THEN pg_get_expr(d.adbin, d.adrelid) END,
      'identity', NULLIF(a.attidentity, '')
    )) FROM pg_attribute a LEFT JOIN pg_attrdef d ON d.adrelid = a.attrelid AND d.adnum = a.attnum
      WHERE a.attrelid = r.oid AND a.attnum > 0 AND NOT a.attisdropped), '{}'::jsonb),
    'primaryKey', (SELECT jsonb_build_object('columns',
      (SELECT jsonb_agg(quote_ident(a.attname) ORDER BY k.ordinality)
       FROM unnest(c.conkey) WITH ORDINALITY k(attnum, ordinality)
       JOIN pg_attribute a ON a.attrelid = r.oid AND a.attnum = k.attnum))
      FROM pg_constraint c WHERE c.conrelid = r.oid AND c.contype = 'p'),
    'uniqueConstraints', COALESCE((SELECT jsonb_object_agg(quote_ident(c.conname), jsonb_build_object('columns',
      (SELECT jsonb_agg(quote_ident(a.attname) ORDER BY k.ordinality)
       FROM unnest(c.conkey) WITH ORDINALITY k(attnum, ordinality)
       JOIN pg_attribute a ON a.attrelid = r.oid AND a.attnum = k.attnum)))
      FROM pg_constraint c WHERE c.conrelid = r.oid AND c.contype = 'u'), '{}'::jsonb),
    'indexes', COALESCE((SELECT jsonb_object_agg(quote_ident(ic.relname), jsonb_build_object(
      'accessMethod', am.amname, 'unique', i.indisunique, 'primary', i.indisprimary,
      'constraintBacked', EXISTS (SELECT 1 FROM pg_constraint c WHERE c.conindid = i.indexrelid AND c.contype IN ('p', 'u')),
      'valid', i.indisvalid, 'ready', i.indisready, 'live', i.indislive, 'immediate', i.indimmediate,
      'predicate', pg_get_expr(i.indpred, i.indrelid), 'definition', pg_get_indexdef(i.indexrelid),
      'keys', (SELECT jsonb_agg(jsonb_build_object(
        'column', a.attname,
        'opclass', quote_ident(opns.nspname) || '.' || quote_ident(op.opcname),
        'collation', CASE WHEN co.oid IS NOT NULL THEN quote_ident(cons.nspname) || '.' || quote_ident(co.collname) END,
        'orderingSupported', COALESCE(op.opcdefault AND (CASE WHEN a.attnum IS NOT NULL THEN i.indcollation[k.n - 1] = a.attcollation ELSE i.indcollation[k.n - 1] = 0 END), false),
        'expression', CASE WHEN a.attname IS NOT NULL THEN quote_ident(a.attname) ELSE pg_get_indexdef(i.indexrelid, k.n, true) END,
        'descending', (i.indoption[k.n - 1] & 1) <> 0,
        'nullsFirst', (i.indoption[k.n - 1] & 2) <> 0
      ) ORDER BY k.n) FROM generate_series(1, i.indnkeyatts) k(n)
        LEFT JOIN pg_attribute a ON a.attrelid = r.oid AND a.attnum = i.indkey[k.n - 1]
        JOIN pg_opclass op ON op.oid = i.indclass[k.n - 1]
        JOIN pg_namespace opns ON opns.oid = op.opcnamespace
        LEFT JOIN pg_collation co ON co.oid = i.indcollation[k.n - 1]
        LEFT JOIN pg_namespace cons ON cons.oid = co.collnamespace)
    )) FROM pg_index i JOIN pg_class ic ON ic.oid = i.indexrelid JOIN pg_am am ON am.oid = ic.relam
       WHERE i.indrelid = r.oid), '{}'::jsonb)
  ) AS value FROM relations r
)
SELECT jsonb_build_object('formatVersion', 2, 'coverage', 'ordering', 'schema', __SCHEMA__,
  'tables', COALESCE((SELECT jsonb_object_agg(name, value) FROM tables), '{}'::jsonb))
WHERE EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = __SCHEMA__);
COMMIT;
