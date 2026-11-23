import { query } from '@example.invalid/db';

await query(`DROP TRIGGER audit_change ON widgets.records CASCADE`);
