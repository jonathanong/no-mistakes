function forward(parameter, key) {
  const escaped = opaque(arguments);
  const removed = delete arguments[key];
  const assigned = (arguments[0] = '/* replacement */ SELECT 1');
  return parameter;
}
const result = forward('SELECT 1', dynamic);
