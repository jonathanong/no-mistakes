import { beginTransaction, type QueryExecutor } from '@example/db';
declare function unconfigured(): any;

// Configured immutable factory results keep their existing positive contract.
const asserted = beginTransaction() as QueryExecutor;
asserted('SELECT 1'); // CLEAR_AS
const angle = <QueryExecutor>beginTransaction();
angle('SELECT 1'); // CLEAR_ANGLE
const satisfied = beginTransaction() satisfies QueryExecutor;
satisfied('SELECT 1'); // CLEAR_SATISFIES

// These receivers have no immutable configured-factory proof.
let mutable = beginTransaction() as QueryExecutor;
mutable('SELECT 1'); // UNKNOWN_MUTABLE
const written = beginTransaction() as QueryExecutor;
written.query = unconfigured();
written('SELECT 1'); // UNKNOWN_WRITE
const opaque = unconfigured() as QueryExecutor;
opaque('SELECT 1'); // UNKNOWN_UNCONFIGURED
