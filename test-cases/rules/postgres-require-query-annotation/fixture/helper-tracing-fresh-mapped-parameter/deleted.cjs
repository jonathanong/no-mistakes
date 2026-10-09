// Definite deletion does not undo an assignment after the opaque update.
function forward(parameter) {
  const escaped = opaque(arguments);
  var parameter = '/* fresh annotation */ SELECT 1';
  delete arguments[0];
  return parameter;
}
const result = forward('placeholder');
