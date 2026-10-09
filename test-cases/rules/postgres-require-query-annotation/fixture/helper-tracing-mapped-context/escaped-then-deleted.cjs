// Deletion disconnects future mapping, but cannot undo a prior opaque update.
function forward(statement) {
  const ignored = opaque(arguments);
  delete arguments[0];
  return statement;
}
const result = forward("/* stale original */ SELECT 1");

function captured(statement) {
  const callback = () => statement;
  const ignored = opaque(arguments);
  delete arguments[0];
  return callback();
}
const capturedResult = captured("/* stale capture */ SELECT 1");

function rebound(callback) {
  const query = sql``;
  const ignored = opaque(arguments);
  var callback = () => query.append("SELECT 1");
  delete arguments[0];
  const invoked = opaque(callback);
  return query;
}
const callbackResult = rebound("placeholder");

// Only the last duplicate parameter has a supplied mapped slot.
function duplicateFirst(statement, statement) {
  const ignored = opaque(arguments);
  delete arguments[0];
  return statement;
}
const duplicateFirstResult = duplicateFirst("first", "/* last value */ SELECT 1");

function duplicateLast(statement, statement) {
  const ignored = opaque(arguments);
  delete arguments[1];
  return statement;
}
const duplicateLastResult = duplicateLast("first", "/* last value */ SELECT 1");

function absentLast(statement, statement) {
  var statement = "/* unmapped local */ SELECT 1";
  const ignored = opaque(arguments);
  delete arguments[0];
  return statement;
}
const absentResult = absentLast("supplied first slot");
