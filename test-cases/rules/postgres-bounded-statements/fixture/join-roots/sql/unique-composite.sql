SELECT i.id
FROM genericitems i
JOIN users u ON u.email = i.email;

SELECT p.id
FROM posts p
JOIN genericitems i
  ON i.tenant_id = p.tenant_id
 AND i.item_no = p.post_no;
