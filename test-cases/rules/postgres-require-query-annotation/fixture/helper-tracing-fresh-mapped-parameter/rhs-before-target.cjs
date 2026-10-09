// Unsupported binding writes happen after RHS reads and modeled mutation.
function mutate(statement) {
  return statement.append('SELECT 1');
}
function forward(statement) {
  const alias = statement;
  const ignored = (statement = mutate(statement));
  return alias;
}
const result = forward(sql``);
