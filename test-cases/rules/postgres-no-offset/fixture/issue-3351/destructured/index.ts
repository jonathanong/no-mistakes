import { query } from 'pg';
const destructured = ({ sql }: { sql: string }) => query(sql);
destructured({ sql: 'SELECT 1 OFFSET 1' });
