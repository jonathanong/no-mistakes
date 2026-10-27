SELECT 1 FROM normalized_emails e WHERE lower(e.email) IS NOT DISTINCT FROM $1;
