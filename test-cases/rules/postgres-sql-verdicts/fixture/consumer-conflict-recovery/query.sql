/* ensureItemCursors */
  INSERT INTO items (family)
  SELECT unnest(enum_range(NULL::item_kinds))
  ON CONFLICT DO NOTHING
