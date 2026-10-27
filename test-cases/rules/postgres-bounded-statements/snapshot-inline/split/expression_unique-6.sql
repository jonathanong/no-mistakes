SELECT 1 FROM normalized_emails AS e WHERE "LOWER"(e.email) = $1;
