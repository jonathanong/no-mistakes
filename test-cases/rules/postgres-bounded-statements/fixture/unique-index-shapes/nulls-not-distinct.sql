-- NULL-safe equality pins nullable keys only when NULLs are not distinct.
SELECT 1 FROM nd_contacts AS n WHERE n.email IS NOT DISTINCT FROM $1;
SELECT 1 FROM plain_contacts AS p WHERE p.email IS NOT DISTINCT FROM $1;
SELECT 1 FROM plain_contacts AS p WHERE p.email = $1;
