WITH updated(batch_id) AS (
  UPDATE orders SET status = 'processed'
  FROM (SELECT id FROM accounts LIMIT 1) AS dropped,
       (SELECT id, code FROM customers LIMIT 1) AS kept,
       (SELECT id FROM invoices LIMIT 1) AS anchor
  WHERE orders.id = kept.id AND orders.account_id = anchor.id
    AND kept.id = dropped.id AND kept.code = anchor.id
  RETURNING account_id
)
UPDATE accounts SET name = 'updated' FROM updated WHERE accounts.id = updated.batch_id;
