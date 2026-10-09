// Sloppy simple parameters share their slots until a known deletion disconnects them.
function forward(statement) {
  const ignored = (arguments[0] = "SELECT 1");
  return statement;
}
const result = forward("/* initial annotation */ SELECT 1");
function disconnected(statement) {
  const removed = delete arguments[0];
  const ignored = unknownMutation(arguments);
  return statement;
}
const preserved = disconnected("/* disconnected parameter */ SELECT 1");
function inherited(statement) {
  const arrow = () => {
    const ignored = unknownMutation(arguments);
    return statement;
  };
  return arrow();
}
const inheritedResult = inherited("/* lexical mapped parameter */ SELECT 1");
function absent(statement) {
  var statement = "/* locally assigned */ SELECT 1";
  const ignored = unknownMutation(arguments);
  return statement;
}
const absentResult = absent();
function duplicateAbsent(statement, statement) {
  var statement = "/* duplicate local */ SELECT 1";
  const ignored = unknownMutation(arguments);
  return statement;
}
const duplicateAbsentResult = duplicateAbsent("SELECT 1");
