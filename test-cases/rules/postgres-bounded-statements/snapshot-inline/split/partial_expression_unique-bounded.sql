SELECT 1 FROM partial_emails AS e WHERE e.email IS NOT NULL AND lower(e.email) = $1;
