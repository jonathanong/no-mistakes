import { runSql } from './helper.mjs';
import { http } from './http.mjs';

// A configured import inside an imported helper executes this SQL.
runSql('SELECT id FROM items OFFSET 2'); // expected: missing SQL annotation
// This same-shape HTTP member must stay outside PostgreSQL executor calls.
http.query('SELECT id FROM items OFFSET 3');
