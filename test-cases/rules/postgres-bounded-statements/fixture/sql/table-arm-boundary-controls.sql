-- Intentionally malformed/unsupported TABLE spellings must never receive delimiter padding.
SELECT 1 UNION ALL TABLE $1;
SELECT 1 UNION ALL TABLE ;
SELECT 1 UNION ALL TABLE public.topics;
SELECT 1 UNION ALL TABLE topics ORDER BY id;
SELECT 1;
