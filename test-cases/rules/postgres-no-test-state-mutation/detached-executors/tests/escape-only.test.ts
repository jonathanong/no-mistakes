import { pool } from '@example/db';
import * as ns from '@example/db';
// An executor exported without a local invocation must retain an analysis obligation.
const { query: namedEscaped } = pool;
export { namedEscaped };
const invalidEscaped = ns.query.bind(otherReceiver);
export { invalidEscaped };
