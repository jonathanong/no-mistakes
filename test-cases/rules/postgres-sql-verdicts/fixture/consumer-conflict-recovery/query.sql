/* ensurePublicationIdentityBridgeCleanupCursors */
  INSERT INTO post_publication_identity_bridge_cleanup_cursors (family)
  SELECT unnest(enum_range(NULL::post_publication_identity_bridge_families))
  ON CONFLICT DO NOTHING
