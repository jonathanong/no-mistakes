import { write } from '@example/db';

// Positive control: a reference is not a call, but calling the alias still reports.
const run = write;
run('SELECT 1');
