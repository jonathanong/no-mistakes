SELECT 1 FROM partial_logins AS p WHERE p.email IS NOT DISTINCT FROM $1;
