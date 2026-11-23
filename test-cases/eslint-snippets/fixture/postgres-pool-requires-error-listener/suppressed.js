import { Pool } from "pg";

const config = { host: "db.example.invalid" };
new Pool(config); // no-mistakes-disable-line postgres-pool-requires-error-listener -- The application owns this listener.
// no-mistakes-disable-next-line no-mistakes/postgres-pool-requires-error-listener -- The application owns this listener.
new Pool(config);
// eslint-disable-next-line no-mistakes/postgres-pool-requires-error-listener -- The application owns this listener.
new Pool(config);

/* eslint-disable no-mistakes/postgres-pool-requires-error-listener -- The application owns these listeners. */
new Pool(config);
/* eslint-enable no-mistakes/postgres-pool-requires-error-listener */
