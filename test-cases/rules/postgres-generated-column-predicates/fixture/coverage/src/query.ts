import { query } from "@example/db";

export function write() {
  return query(`
    CREATE TRIGGER touch AFTER INSERT ON orders
    FOR EACH ROW EXECUTE FUNCTION touch();
    INSERT INTO orders (id) VALUES ($1) RETURNING *;
  `);
}

export function load(table: string) {
  return query(`SELECT 1 FROM ${table}`);
}
