SELECT 1 FROM normalized_emails e(email) WHERE lower(e.email) = $1;
