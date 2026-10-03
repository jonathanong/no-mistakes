import { query } from '@example/db';
export const page = () => query(`SELECT id FROM orders OFFSET 1`);
