import { helper } from '@example/db';
// This imported member is advisory-only; the unknown call must remain visible.
helper.query('SELECT 1 OFFSET 5');
