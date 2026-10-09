// Escaping a primitive value cannot replace its mapped parameter binding.
function forward(parameter) {
  const escaped = opaque(arguments);
  var parameter = '/* fresh annotation */ SELECT 1';
  const ignored = opaque(parameter);
  return parameter;
}
const result = forward('placeholder');
