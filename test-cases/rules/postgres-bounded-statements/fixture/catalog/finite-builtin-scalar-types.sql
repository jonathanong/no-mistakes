-- Only non-array base/range/multirange types prove finite constructor leaves.
SELECT typname FROM pg_catalog.pg_type WHERE typnamespace = 'pg_catalog'::regnamespace AND typtype IN ('b', 'r', 'm') AND typcategory <> 'A' ORDER BY typname;
