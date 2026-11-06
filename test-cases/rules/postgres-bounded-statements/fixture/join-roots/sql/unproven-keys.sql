SELECT p.id
FROM posts p
JOIN genericitems i ON i.tenant_id = p.tenant_id;

SELECT i.id
FROM genericitems i
JOIN users u ON u.email IS NOT DISTINCT FROM i.email;

SELECT p.id
FROM posts p
JOIN users u ON u.id = coalesce(p.user_id, '00000000-0000-0000-0000-000000000000'::uuid);

SELECT i.id
FROM genericitems i
JOIN users u ON u.email = i.email;

SELECT p.id
FROM posts p
JOIN users u ON u.id = $1;
