-- Expression keys require the same expression on the bounded side.
SELECT 1 FROM normalized_emails AS e WHERE lower(e.email) = $1;
SELECT 1 FROM directories AS d JOIN normalized_emails AS e ON lower(e.email) = lower(d.email) WHERE d.id = $1;
-- Unquoted function names fold case; quoted uppercase names identify another function.
SELECT 1 FROM normalized_emails AS e WHERE LOWER(e.email) = $1;
