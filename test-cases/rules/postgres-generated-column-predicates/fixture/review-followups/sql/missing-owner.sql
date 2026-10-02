SELECT o.id FROM orders o JOIN missing m ON true WHERE created_at > $1;
