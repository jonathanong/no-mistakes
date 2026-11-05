SELECT i.id
FROM genericitems i
JOIN users u ON u.handle = i.handle;

SELECT p.id
FROM posts p
JOIN genericitems i
  ON i.tenant_id = p.tenant_id
 AND i.item_no = p.post_no;
