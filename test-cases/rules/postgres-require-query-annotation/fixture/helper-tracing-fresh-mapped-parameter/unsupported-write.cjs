// Unsupported nested assignment must cancel the fresh parameter proof.
function forward(parameter) {
  const escaped = opaque(arguments);
  var parameter = '/* fresh annotation */ SELECT 1';
  const ignored = condition ? (parameter = 'SELECT 1') : false;
  return parameter;
}
const result = forward('placeholder');
