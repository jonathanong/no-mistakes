// The short-circuit path keeps the slot reachable by the opaque consumer.
function forward(parameter) {
  const ignored = condition && delete arguments[0];
  const escaped = opaque(arguments);
  return parameter;
}
const result = forward('/* logical possible deletion */ SELECT 1');
