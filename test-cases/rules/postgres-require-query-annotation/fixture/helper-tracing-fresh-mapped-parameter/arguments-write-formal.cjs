function forward(parameter) {
  const ignored = (arguments = ["SELECT 1"]);
  return parameter;
}
// Rebinding the invocation object does not mutate its former builder slot.
const result = forward(sql`/* detached formal */ SELECT 1`);
