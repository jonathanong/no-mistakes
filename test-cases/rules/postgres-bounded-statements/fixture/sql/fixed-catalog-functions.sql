-- Only explicit catalog identity is trusted across the supported PostgreSQL versions.
SELECT a.email FROM pg_catalog.pg_stat_get_recovery_prefetch() p JOIN accounts a ON a.email = p.stats_reset::text;
SELECT a.email FROM pg_stat_get_recovery_prefetch() p JOIN accounts a ON a.email = p.stats_reset::text;
SELECT a.email FROM public.pg_stat_get_recovery_prefetch() p JOIN accounts a ON a.email = p.stats_reset::text;
SELECT a.email FROM pg_catalog.pg_stat_get_recovery_prefetch($1) p JOIN accounts a ON a.email = p.stats_reset::text;
SELECT a.email FROM pg_catalog.pg_ls_dir($1) p JOIN accounts a ON a.email = p.name;
SELECT a.email FROM pg_catalog.unnest($1) p JOIN accounts a ON a.email = p.unnest;
