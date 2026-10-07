-- Generic parser shapes remain explicitly unsupported in PostgreSQL projections.
INSERT INTO accounts TABLE public.source_accounts;
INSERT INTO accounts VALUES (1) ON DUPLICATE KEY UPDATE id = 1;
