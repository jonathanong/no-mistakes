import { query } from '@example/db';
// Malformed executable SQL must not be treated as proven harmless.
query('SELECT FROM');
