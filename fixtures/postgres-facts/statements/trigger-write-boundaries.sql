DELETE FROM orders, other_orders;
SELECT set_config();
SELECT set_config((SELECT setting_name FROM settings), 'replica', true);
