import { read, write, type QueryExecutor, type QueryOptions } from '@example/db';
function choices(left: QueryExecutor, right: QueryExecutor, options: QueryOptions, unknown: unknown) {
  (read || write)('SELECT 1'); // KNOWN_IMPORTED_OR
  (options.query ?? read)('SELECT 2'); // KNOWN_MEMBER_LEFT
  (read ?? options.query)('SELECT 3'); // KNOWN_MEMBER_RIGHT
  (options?.query ?? right)('SELECT 4'); // KNOWN_OPTIONAL_MEMBER
  (left || (options.query ?? right))('SELECT 5'); // KNOWN_NESTED
  (options.other ?? read)('SELECT 6'); // UNKNOWN_OTHER_MEMBER
  (options['query'] ?? read)('SELECT 7'); // UNKNOWN_COMPUTED_MEMBER
  (read && write)('SELECT 8'); // UNKNOWN_AND
  (unknown ?? read)('SELECT 9'); // UNKNOWN_OPERAND
  const called = left ?? right;
  called('SELECT 10'); // KNOWN_CALLED_ALIAS
  const unused = left ?? right; // QUIET_UNUSED_ALIAS
  const { ignored } = left ?? right; // QUIET_PATTERN_ALIAS
}
function forward(query: QueryExecutor) { query('SELECT 11'); } // KNOWN_TYPED_FORWARD_BODY
function quiet(options: QueryOptions) { return options.timeout; }
const quietArrow = (options: QueryOptions) => options.timeout;
const quietFunction = function(options: QueryOptions) { return options.timeout; };
function passed(left: QueryExecutor, right: QueryExecutor) {
  const chosen = left || right; // QUIET_PASSED_ALIAS
  forward(chosen);
  forward(left ?? right); // QUIET_COMPATIBLE_ARGUMENT
}
read('SELECT 12', { query: read ?? write }); // KNOWN_CHOICE_OPTIONS
read('SELECT 13', { query: read }); // KNOWN_NAMED_OPTIONS
read('SELECT 14', { other: 1, ...{}, query: read }); // KNOWN_SPREAD_OPTIONS
read('SELECT 15', 1); // KNOWN_PRIMITIVE_OPTIONS
quiet({ query: read }); // QUIET_OPTIONS
quietArrow({ query: read ?? write }); // QUIET_ARROW_OPTIONS
quietFunction({ query: write }); // QUIET_FUNCTION_OPTIONS
