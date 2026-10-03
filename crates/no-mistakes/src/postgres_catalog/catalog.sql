-- One read-only query for both coverages. Rust substitutes __SCHEMA__ (an SQL string literal),
-- __COVERAGE__ (a string literal) and __COMPLETE__ (true or false). Facts that only a complete
-- catalog carries sit behind CASE WHEN __COMPLETE__, so the two coverages cannot diverge: they
-- run the same relation selection and the same expressions for every shared field.
WITH selected AS (
  SELECT oid FROM pg_namespace WHERE nspname = __SCHEMA__
), extension_members AS (
  SELECT classid, objid FROM pg_depend WHERE deptype = 'e'
), relations AS (
  -- The one relation-selection policy. Partition leaves roll up into their parent, so only the
  -- partitioned parent appears and nothing cloned onto a leaf is a separate entry. Sequences,
  -- TOAST, indexes, composite types and foreign tables (unsupported) are other relkinds, and
  -- temporary and extension-owned relations never describe the application schema.
  SELECT c.oid, c.relname, c.relkind FROM pg_class c JOIN selected s ON s.oid = c.relnamespace
  WHERE c.relkind IN ('r', 'p', 'v', 'm') AND c.relpersistence <> 't' AND NOT c.relispartition
    AND NOT EXISTS (SELECT 1 FROM extension_members e
                    WHERE e.classid = 'pg_class'::regclass AND e.objid = c.oid)
), constraints AS (
  -- conparentid = 0 drops the per-partition clones PostgreSQL adds to a table whose foreign
  -- key references a partitioned table. contype 'n' (PostgreSQL 18 NOT NULL) is not a check.
  SELECT c.conrelid, quote_ident(c.conname) AS name, c.contype, c.convalidated AS validated,
    (SELECT jsonb_agg(a.attname ORDER BY k.ordinality)
     FROM unnest(c.conkey) WITH ORDINALITY k(attnum, ordinality)
     JOIN pg_attribute a ON a.attrelid = c.conrelid AND a.attnum = k.attnum) AS columns,
    (SELECT jsonb_agg(quote_ident(a.attname) ORDER BY k.ordinality)
     FROM unnest(c.conkey) WITH ORDINALITY k(attnum, ordinality)
     JOIN pg_attribute a ON a.attrelid = c.conrelid AND a.attnum = k.attnum) AS sql_columns,
    (SELECT jsonb_agg(a.attname ORDER BY k.ordinality)
     FROM unnest(c.confkey) WITH ORDINALITY k(attnum, ordinality)
     JOIN pg_attribute a ON a.attrelid = c.confrelid AND a.attnum = k.attnum) AS referenced_columns,
    (SELECT CASE WHEN f.relnamespace = (SELECT oid FROM selected) THEN quote_ident(f.relname)
            ELSE quote_ident(n.nspname) || '.' || quote_ident(f.relname) END
     FROM pg_class f JOIN pg_namespace n ON n.oid = f.relnamespace
     WHERE f.oid = c.confrelid) AS referenced_table,
    CASE c.confdeltype WHEN 'a' THEN 'NO ACTION' WHEN 'r' THEN 'RESTRICT' WHEN 'c' THEN 'CASCADE'
      WHEN 'n' THEN 'SET NULL' WHEN 'd' THEN 'SET DEFAULT' END AS on_delete,
    CASE c.confupdtype WHEN 'a' THEN 'NO ACTION' WHEN 'r' THEN 'RESTRICT' WHEN 'c' THEN 'CASCADE'
      WHEN 'n' THEN 'SET NULL' WHEN 'd' THEN 'SET DEFAULT' END AS on_update,
    pg_get_constraintdef(c.oid) AS definition
  FROM pg_constraint c JOIN relations r ON r.oid = c.conrelid
  WHERE c.conparentid = 0 AND c.contype IN ('p', 'u', 'f', 'c')
), tables AS (
  SELECT quote_ident(r.relname) AS name, jsonb_build_object(
    'relationKind', CASE r.relkind WHEN 'p' THEN 'partitioned table' ELSE 'table' END,
    'columns', COALESCE((SELECT jsonb_object_agg(a.attname, jsonb_build_object(
      'dataType', format_type(a.atttypid, a.atttypmod), 'nullable', NOT a.attnotnull,
      'ordinalPosition', a.position,
      'defaultExpression', CASE WHEN a.attgenerated = '' THEN pg_get_expr(d.adbin, d.adrelid) END,
      'generated', CASE a.attgenerated WHEN 's' THEN 'stored' WHEN 'v' THEN 'virtual' END,
      'generatedExpression', CASE WHEN a.attgenerated <> '' THEN pg_get_expr(d.adbin, d.adrelid) END,
      'identity', NULLIF(a.attidentity, '')
    ) || CASE WHEN __COMPLETE__ THEN jsonb_build_object('comment', col_description(r.oid, a.attnum))
              ELSE '{}'::jsonb END)
      FROM (SELECT attrelid, attnum, attname, atttypid, atttypmod, attnotnull, attgenerated,
              attidentity, row_number() OVER (ORDER BY attnum) AS position
            FROM pg_attribute WHERE attrelid = r.oid AND attnum > 0 AND NOT attisdropped) a
      LEFT JOIN pg_attrdef d ON d.adrelid = a.attrelid AND d.adnum = a.attnum), '{}'::jsonb),
    'primaryKey', (SELECT jsonb_build_object('columns', c.columns) FROM constraints c
                   WHERE c.conrelid = r.oid AND c.contype = 'p'),
    'uniqueConstraints', COALESCE((SELECT jsonb_object_agg(c.name,
      jsonb_build_object('columns', c.sql_columns)) FROM constraints c
      WHERE c.conrelid = r.oid AND c.contype = 'u'), '{}'::jsonb),
    'indexes', COALESCE((SELECT jsonb_object_agg(quote_ident(ic.relname), jsonb_build_object(
      'accessMethod', am.amname, 'unique', i.indisunique, 'primary', i.indisprimary,
      'constraintBacked', EXISTS (SELECT 1 FROM pg_constraint c WHERE c.conindid = i.indexrelid AND c.contype IN ('p', 'u', 'x')),
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
  ) || CASE WHEN __COMPLETE__ THEN jsonb_build_object(
    'comment', obj_description(r.oid, 'pg_class'),
    'physicalPartition', CASE WHEN r.relkind = 'p'
      THEN jsonb_build_object('key', pg_get_partkeydef(r.oid)) END,
    'foreignKeys', COALESCE((SELECT jsonb_object_agg(c.name, jsonb_build_object(
      'columns', c.columns, 'referencedTable', c.referenced_table,
      'referencedColumns', c.referenced_columns, 'onDelete', c.on_delete,
      'onUpdate', c.on_update, 'validated', c.validated)) FROM constraints c
      WHERE c.conrelid = r.oid AND c.contype = 'f'), '{}'::jsonb),
    'checkConstraints', COALESCE((SELECT jsonb_object_agg(c.name, jsonb_build_object(
      'definition', c.definition, 'validated', c.validated)) FROM constraints c
      WHERE c.conrelid = r.oid AND c.contype = 'c'), '{}'::jsonb),
    'triggers', COALESCE((SELECT jsonb_object_agg(quote_ident(t.tgname),
      jsonb_build_object('definition', pg_get_triggerdef(t.oid))) FROM pg_trigger t
      WHERE t.tgrelid = r.oid AND NOT t.tgisinternal), '{}'::jsonb)
  ) ELSE '{}'::jsonb END AS value FROM relations r WHERE r.relkind IN ('r', 'p')
), functions AS (
  -- pg_get_functiondef fails on aggregates and does not describe window functions: prokind f/p only.
  SELECT quote_ident(p.proname) || '(' || pg_get_function_identity_arguments(p.oid) || ')' AS key,
    jsonb_build_object('definition', pg_get_functiondef(p.oid)) AS value
  FROM pg_proc p JOIN selected s ON s.oid = p.pronamespace
  WHERE p.prokind IN ('f', 'p') AND NOT EXISTS (SELECT 1 FROM extension_members e
                                                WHERE e.classid = 'pg_proc'::regclass AND e.objid = p.oid)
), enums AS (
  -- Keyed by the type as a column renders it (format_type under the same search_path), so a
  -- column's element type always finds its enum, including one named like a pg_catalog type.
  -- A zero-label enum is legal, and jsonb_agg of no rows is NULL, so it is coalesced to [].
  SELECT format_type(t.oid, NULL) AS key, jsonb_build_object('values', COALESCE((SELECT
    jsonb_agg(v.enumlabel ORDER BY v.enumsortorder) FROM pg_enum v WHERE v.enumtypid = t.oid),
    '[]'::jsonb)) AS value
  FROM pg_type t JOIN selected s ON s.oid = t.typnamespace
  WHERE t.typtype = 'e' AND NOT EXISTS (SELECT 1 FROM extension_members e
                                        WHERE e.classid = 'pg_type'::regclass AND e.objid = t.oid)
), views AS (
  SELECT quote_ident(r.relname) AS key, jsonb_build_object('materialized', r.relkind = 'm',
    'definition', pg_get_viewdef(r.oid, true), 'comment', obj_description(r.oid, 'pg_class')) AS value
  FROM relations r WHERE r.relkind IN ('v', 'm')
)
SELECT jsonb_build_object('formatVersion', 2, 'coverage', __COVERAGE__, 'schema', __SCHEMA__,
  'tables', COALESCE((SELECT jsonb_object_agg(name, value) FROM tables), '{}'::jsonb))
  || CASE WHEN __COMPLETE__ THEN jsonb_build_object(
    'functions', COALESCE((SELECT jsonb_object_agg(key, value) FROM functions), '{}'::jsonb),
    'enums', COALESCE((SELECT jsonb_object_agg(key, value) FROM enums), '{}'::jsonb),
    'views', COALESCE((SELECT jsonb_object_agg(key, value) FROM views), '{}'::jsonb)
  ) ELSE '{}'::jsonb END
WHERE EXISTS (SELECT 1 FROM selected);
