SELECT 1 FROM partial_emails AS e WHERE lower(e.email) IS NOT DISTINCT FROM $1;
