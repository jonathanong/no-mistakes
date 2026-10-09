// Only the last duplicate parameter maps to its argument slot in sloppy scripts.
function first(statement, statement) {
  var statement = "replacement";
  return arguments[0];
}
function last(statement, statement) {
  var statement = "replacement";
  return arguments[1];
}
const original = first("/* retained first */ SELECT 1", "/* mapped last */ SELECT 1");
const result = last("/* retained first */ SELECT 1", "/* mapped last */ SELECT 1");
