import pg from 'pg';
const { Pool } = pg;
export { Pool };
export function open() { return new Pool(); }
