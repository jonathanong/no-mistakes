import { beginTransaction } from '@example/db';
const transaction = beginTransaction();
mutate();
transaction.query({ resource: 'http' });
transaction('SELECT direct late');
// A later declaration can mutate the member before its earlier source-site use.
function mutate() { transaction.query = (_request: object) => undefined; }
