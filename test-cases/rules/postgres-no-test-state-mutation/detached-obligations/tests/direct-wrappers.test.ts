import * as db from '@example/db';
(db.query)('DROP TABLE member_call');
const { query } = db;
(query as typeof query)('DROP TABLE alias_call');
