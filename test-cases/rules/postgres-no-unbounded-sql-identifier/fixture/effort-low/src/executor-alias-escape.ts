import { query as q } from '@example/db';
const run = q;
unknownHelper(run);
// An unused alias has no executor invocation or escape to report.
const unused = q;
