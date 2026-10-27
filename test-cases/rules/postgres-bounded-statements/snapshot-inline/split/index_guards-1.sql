-- Alias lists must not reinterpret the physical key or an expression target.
SELECT 1 FROM nd_contacts n(label, email) WHERE n.email IS NOT DISTINCT FROM $1;
