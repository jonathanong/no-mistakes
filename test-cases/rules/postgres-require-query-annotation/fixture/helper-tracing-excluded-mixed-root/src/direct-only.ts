import { query } from '@example/db';
// This excluded executor has no path to an included executor and stays pruned.
query('SELECT * FROM also_unannotated');
