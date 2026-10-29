import { format } from '@example/db';
// Binding an unselected function never establishes query-method provenance.
const fmt = format.bind(null);
fmt('hello');
const alias = fmt;
alias('DROP TABLE ordinary_function_argument');
let assigned;
assigned = format.bind(null, 'prefix');
assigned('hello');
