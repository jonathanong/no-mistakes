-- lower(email) = $1 implies email is not null, so the partial unique index bounds the scan.
SELECT 1 FROM partial_emails AS e WHERE lower(e.email) = $1;
