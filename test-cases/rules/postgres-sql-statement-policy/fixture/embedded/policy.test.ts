import { query as execute, sql, beginTransaction, type TransactionQuery } from '@example/db';
execute(sql`ALTER TABLE orders ADD CONSTRAINT reject_writes CHECK (false)`);
const transaction = beginTransaction();
transaction(sql`CREATE VIEW orders_view AS SELECT * FROM orders`);
function typed(execute: TransactionQuery) {
  execute(sql`TRUNCATE orders`);
}
execute(sql`INSERT INTO orders (id) VALUES (1)`);
