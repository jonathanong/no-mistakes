const database = require("@app/db");
// The analyzers' established conventional global sql tag is intentionally unbound.

function forward(first, ignored) { return first; }
async function promised(value) { return value; }
function disposable() { return 0; }

function savedCallback(comparison, object) {
  return () => {
    // Callback reads must retain references inside both scalar and object values.
    const table = { comparison, object };
    const selected = table[key];
    database.query(sql`${comparison}`); // callback comparison bind
    database.query(sql`${selected}`); // callback selected object bind
    database.query(sql`${object.query}`); // callback object fragment
  };
}

function lifetime() {
  const statement = sql``;
  const promise = promised(statement);
  const comparison = statement === 0;
  const callbackComparison = (() => database.query("SELECT 9")) === 0; // retained comparison callback
  const object = { [0]: statement, query: statement, callback: callbackComparison, aliases: [comparison] };
  // The later operand updates live references without changing scalar bind results.
  const refreshed = forward(comparison, promise, statement.append("SELECT 10"));
  database.query(sql`${refreshed}`); // refreshed comparison bind
  database.query(statement); // mutation through retained containers
  const ignored = disposable();
  // Discarded helper frames must not steal callback or object capture identities.
  return choose
    ? [savedCallback(callbackComparison, object), callbackComparison, object]
    : [savedCallback(callbackComparison, object), callbackComparison, object];
}
opaque(choose ? lifetime() : []);

function conditionalContainers(comparison, object, statement) {
  // Sloppy argument slots update the formals; both not-taken versions survive.
  const first = choose ? (arguments[0] = comparison) : (arguments[0] = 1);
  const second = choose ? (arguments[1] = object) : (arguments[1] = 2);
  statement.append("SELECT 11");
  database.query(statement); // joined references retain builder mutation
  database.query(sql`${comparison} ${object}`); // joined scalar containers remain binds
  database.query(sql`${object.query}`); // joined object fragment
  database.query(arguments[1].query); // possible slot property
  const escaped = opaque(comparison);
  database.query(statement); // joined comparison escape
}
const joinedStatement = sql``;
conditionalContainers(joinedStatement === 0, { query: joinedStatement }, joinedStatement);
