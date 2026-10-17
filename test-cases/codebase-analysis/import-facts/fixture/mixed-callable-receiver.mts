import { db, read } from '@vendor/mixed-callable-receiver';
let callback;
callback = function preserved() { return callback; };
// Retained callable assignments must not replace the real receiver proof.
db.query = read;
db.query();
