import { query } from '@data-stores/psql';
export const page = () => query(`SELECT id FROM orders OFFSET 1`);
