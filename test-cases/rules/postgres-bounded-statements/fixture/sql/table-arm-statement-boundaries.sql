-- Preserve both TABLE nodes and their original quoted/folded identity.
SELECT id FROM safe UNION ALL TABLE "Topics";
SELECT id FROM safe UNION ALL TABLE topics;
SELECT id FROM safe UNION DISTINCT TABLE public.topics;
SELECT id FROM safe;
