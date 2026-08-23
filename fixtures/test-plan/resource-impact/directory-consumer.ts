import { readdirSync } from 'node:fs';

// `003-added.sql` is kept in the saved fixture so this call exercises a file
// becoming tracked without constructing it during the test run.
export const migrationFiles = readdirSync('migrations');
