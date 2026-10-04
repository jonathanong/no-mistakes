-- Independent component quoting must retain exact schema/relation identity.
TABLE "Tenant".Accounts;
TABLE Tenant."Accounts";
SELECT NULL UNION ALL TABLE "Tenant".Accounts;
SELECT NULL UNION ALL TABLE Tenant."Accounts";
SELECT NULL UNION ALL TABLE "Tenant.Name".Accounts;
SELECT NULL UNION ALL TABLE Tenant."Account.Name";
