-- Sole-column legacy catalogs prove position without inventing ordinal metadata.
SELECT * FROM tokens t(id) WHERE t.id = $1;
SELECT * FROM tokens t(renamed) WHERE t.renamed = $1;
SELECT * FROM tokens t(renamed) WHERE t.renamed IS NOT DISTINCT FROM $1;
SELECT * FROM nullable_token t(renamed) WHERE t.renamed IS NOT DISTINCT FROM $1;
SELECT * FROM ambiguous_tokens t(id) WHERE t.id = $1;
SELECT * FROM ambiguous_tokens t(renamed) WHERE t.renamed = $1;
SELECT * FROM unkeyed_token t(ctid) WHERE t.ctid = $1;
SELECT * FROM tokens t(renamed) WHERE t.ctid = $1;
SELECT * FROM tokens a JOIN tokens t(renamed) ON a.id = ANY(ARRAY[t.renamed, $1]) WHERE t.renamed = $2;
SELECT * FROM tokens a JOIN stored_tokens t(renamed) ON a.id = ANY(ARRAY[t.renamed]::uuid[]) WHERE t.renamed = $2;
