UPDATE orders SET computed = 1;
UPDATE orders SET discarded = 1;
UPDATE bare_orders SET computed = 1;
UPDATE transaction_orders SET computed = 1;
UPDATE no_chain_orders SET computed = 1;
