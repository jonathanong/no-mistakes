SELECT 1 FROM plain_contacts AS p WHERE p.email IS NOT DISTINCT FROM $1;
