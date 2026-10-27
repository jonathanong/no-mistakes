SELECT 1 FROM partial_emails AS e WHERE e.email IS NULL AND lower(e.email) IS NOT DISTINCT FROM NULL;
