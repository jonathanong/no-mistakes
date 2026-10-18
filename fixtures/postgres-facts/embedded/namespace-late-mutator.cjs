const db = require('@example/db');
mutate();
db.query({ resource: 'http' });
// The function is declared later but runs before the apparent SQL call.
function mutate() { db.query = (_request) => undefined; }
