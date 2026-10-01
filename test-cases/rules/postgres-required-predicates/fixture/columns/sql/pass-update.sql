UPDATE orders SET status = 'paid' WHERE account_id = $1 AND id = $2;
