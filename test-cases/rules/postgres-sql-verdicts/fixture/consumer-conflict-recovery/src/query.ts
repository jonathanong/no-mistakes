import { query } from '@example/db'

// The set-returning enum function makes this targetless upsert potentially multi-row.
query(`/* ensureItemCursors */
  INSERT INTO items (family)
  SELECT unnest(enum_range(NULL::item_kinds))
  ON CONFLICT DO NOTHING`)
