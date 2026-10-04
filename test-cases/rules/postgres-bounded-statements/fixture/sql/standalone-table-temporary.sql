-- Standalone TABLE statements must keep quote and schema identity after preceding DDL.
CREATE TEMP TABLE "Accounts"(id uuid);
TABLE "Accounts";
TABLE Accounts;
TABLE public."Accounts";
TABLE public.accounts;
TABLE pg_temp."Accounts";
TABLE ONLY "Accounts";
