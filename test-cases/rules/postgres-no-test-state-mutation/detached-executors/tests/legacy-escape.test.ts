import { pool, query as direct } from '@example/db';
import * as ns from '@example/db';
// Invalid, uninvoked binds must not consume the existing low projection's escape advice.
const bad = pool.query.bind(direct);
const namespaceBad = ns.query.bind(otherReceiver);
