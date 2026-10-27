SELECT 1 FROM normalized_emails AS e WHERE lower(e.email)::varchar = $1;
