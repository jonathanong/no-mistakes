-- lock-reason: prevents: double refund; writer: refund worker retry
SELECT id FROM refunds WHERE id = $1 FOR UPDATE;

SELECT id FROM orders WHERE id = $1 FOR SHARE;
