function forward(parameter) {
  const escaped = opaque(arguments);
  const assigned = (arguments[0] = 'SELECT 1');
  const removed = delete arguments[0];
  // Deletion disconnects future updates, not the known value written before it.
  return parameter;
}
const result = forward('/* original annotation */ SELECT 1');
