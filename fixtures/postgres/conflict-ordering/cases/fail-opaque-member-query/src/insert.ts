declare function assembleWriter(): string;
// The configured CommonJS module proves this receiver is a SQL executor.
const client = require('@example/db');

client.query(assembleWriter());
